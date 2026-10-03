use rustc_hir as hir;
use rustc_hir::intravisit::FnKind;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;
use crate::performance::allocation_in_loop;
use crate::test_context;

const AMBIENT_ENV_LINT: &str = "ambient_env";
const AMBIENT_RANDOM_LINT: &str = "ambient_random";
const GLOBAL_STATE_IN_CORE_LINT: &str = "global_state_in_core";
const IMPURE_CALL_IN_CORE_LINT: &str = "impure_call_in_core";
const INTERIOR_MUTABILITY_IN_CORE_LINT: &str = "interior_mutability_in_core";
const LOGGING_IN_CORE_LINT: &str = "logging_in_core";
const MISSING_CONST_FN_LINT: &str = "missing_const_fn";
const MUTATING_INPUT_IN_PURE_LINT: &str = "mutating_input_in_pure";
const NONDETERMINISTIC_OUTPUT_LINT: &str = "nondeterministic_output";
const PURE_ATTR_CONTRACT_LINT: &str = "pure_attr_contract";
const ZERO_ALLOC_ATTR_CONTRACT_LINT: &str = "zero_alloc_attr_contract";
const RESOURCE_RETURN_FROM_CORE_LINT: &str = "resource_return_from_core";
const SPAWN_IN_CORE_LINT: &str = "spawn_in_core";

const ENV_CALLS: &[&str] = &[
    "args",
    "args_os",
    "current_dir",
    "remove_var",
    "set_current_dir",
    "set_var",
    "temp_dir",
    "var",
    "var_os",
    "vars",
    "vars_os",
];
const RANDOM_CALL_PATTERNS: &[&str] = &[
    "fastrand::",
    "getrandom::fill(",
    "getrandom::getrandom(",
    "rand::random(",
    "rand::rng(",
    "rand::thread_rng(",
    "uuid::Uuid::new_v4(",
    "Uuid::new_v4(",
];
const FILESYSTEM_CALL_PATTERNS: &[&str] = &[
    "std::fs::create_dir(",
    "std::fs::create_dir_all(",
    "std::fs::read(",
    "std::fs::read_dir(",
    "std::fs::read_to_string(",
    "std::fs::remove_file(",
    "std::fs::write(",
    "std::fs::File::create(",
    "std::fs::File::open(",
    "fs::read(",
    "fs::read_to_string(",
    "fs::write(",
    "File::open(",
    "File::create(",
];
const IO_CALL_PATTERNS: &[&str] = &["std::io::stdin(", "std::io::stdout(", "std::io::stderr("];
const NET_CALL_PATTERNS: &[&str] = &[
    "std::net::TcpListener::bind(",
    "std::net::TcpStream::connect(",
    "std::net::UdpSocket::bind(",
    "TcpListener::bind(",
    "TcpStream::connect(",
    "UdpSocket::bind(",
];
const PROCESS_CALL_PATTERNS: &[&str] = &[
    "std::process::Command::new(",
    "std::process::exit(",
    "Command::new(",
    "process::Command::new(",
];
const TIME_CALL_PATTERNS: &[&str] = &[
    "Instant::now(",
    "Local::now(",
    "SystemTime::now(",
    "Utc::now(",
    "std::time::Instant::now(",
    "std::time::SystemTime::now(",
];
const RUNTIME_CALL_PATTERNS: &[&str] = &["tokio::runtime::", "Runtime::new(", "Builder::new_current_thread("];
const LOGGING_CALL_PATTERNS: &[&str] = &[
    "dbg!(",
    "eprintln!(",
    "log::debug!(",
    "log::error!(",
    "log::info!(",
    "log::trace!(",
    "log::warn!(",
    "println!(",
    "tracing::debug!(",
    "tracing::error!(",
    "tracing::event!(",
    "tracing::info!(",
    "tracing::trace!(",
    "tracing::warn!(",
];
const SPAWN_CALL_PATTERNS: &[&str] = &[
    "async_std::task::spawn(",
    "std::thread::spawn(",
    "thread::spawn(",
    "tokio::spawn(",
    "tokio::task::spawn(",
];
const GLOBAL_STATE_PATTERNS: &[&str] = &[
    "static mut ",
    "thread_local!(",
    "AtomicBool",
    "AtomicI",
    "AtomicPtr",
    "AtomicU",
    "LazyLock<",
    "Mutex<",
    "OnceLock<",
    "RefCell<",
    "RwLock<",
    "UnsafeCell<",
];
const INTERIOR_MUTABILITY_PATTERNS: &[&str] = &[
    "AtomicBool",
    "AtomicI",
    "AtomicPtr",
    "AtomicU",
    "Cell<",
    "Cell::new(",
    "LazyLock<",
    "LazyLock::new(",
    "Mutex<",
    "Mutex::new(",
    "OnceLock<",
    "OnceLock::new(",
    "RefCell<",
    "RefCell::new(",
    "RwLock<",
    "RwLock::new(",
    "UnsafeCell<",
    "UnsafeCell::new(",
];
const MUTATING_PARAM_PATTERNS: &[&str] = &[
    "&mut ",
    "*mut ",
    "AtomicBool",
    "AtomicI",
    "AtomicPtr",
    "AtomicU",
    "Cell<",
    "Mutex<",
    "RefCell<",
    "RwLock<",
    "UnsafeCell<",
];
const RESOURCE_RETURN_PATTERNS: &[&str] = &[
    "Child",
    "Client",
    "Command",
    "Connection",
    "File",
    "JoinHandle",
    "Receiver<",
    "Sender<",
    "Socket",
    "TcpListener",
    "TcpStream",
    "UdpSocket",
    "UnixListener",
    "UnixStream",
];
const RESOURCE_DESCRIPTION_ALLOW_PATTERNS: &[&str] = &[
    "CommandDescription",
    "CommandPlan",
    "CommandSpec",
    "Description",
    "Descriptor",
    "Plan",
    "Port",
    "Request",
    "Spec",
];
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectClass {
    Env,
    Filesystem,
    Io,
    Logging,
    Net,
    Process,
    Random,
    Runtime,
    Spawn,
    Time,
}

impl EffectClass {
    fn label(self) -> &'static str {
        match self {
            Self::Env => "environment",
            Self::Filesystem => "filesystem",
            Self::Io => "standard IO",
            Self::Logging => "logging",
            Self::Net => "network",
            Self::Process => "process",
            Self::Random => "randomness",
            Self::Runtime => "runtime construction",
            Self::Spawn => "task spawning",
            Self::Time => "time",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PureViolation {
    DirectEffect(EffectClass),
    GlobalState,
    InteriorMutability,
    MutatingInput,
    Panic,
    ResourceReturn,
}

impl PureViolation {
    fn label(self) -> &'static str {
        match self {
            Self::DirectEffect(effect) => effect.label(),
            Self::GlobalState => "global state",
            Self::InteriorMutability => "interior mutability",
            Self::MutatingInput => "mutating input",
            Self::Panic => "panic",
            Self::ResourceReturn => "resource return",
        }
    }
}

declare_tool_lint! {
    /// Flags direct process environment reads or mutations in production code.
    pub tigerstyle::AMBIENT_ENV,
    Warn,
    "direct ambient process environment access"
}

declare_tool_lint! {
    /// Flags direct ambient randomness and non-deterministic ID generation.
    pub tigerstyle::AMBIENT_RANDOM,
    Warn,
    "direct ambient randomness source"
}

declare_tool_lint! {
    /// Flags process-wide mutable or lazy state declarations in configured core scopes.
    pub tigerstyle::GLOBAL_STATE_IN_CORE,
    Warn,
    "global mutable state in configured core scope"
}

declare_tool_lint! {
    /// Flags direct platform-effect calls in configured core scopes.
    pub tigerstyle::IMPURE_CALL_IN_CORE,
    Warn,
    "platform effect call in configured core scope"
}

declare_tool_lint! {
    /// Flags interior mutability and synchronization primitives in configured core scopes.
    pub tigerstyle::INTERIOR_MUTABILITY_IN_CORE,
    Warn,
    "interior mutability in configured core scope"
}

declare_tool_lint! {
    /// Flags logging and stdout/stderr emission in configured core scopes.
    pub tigerstyle::LOGGING_IN_CORE,
    Warn,
    "logging or console output in configured core scope"
}

declare_tool_lint! {
    /// Advises when a private helper is conservatively eligible to become `const fn`.
    pub tigerstyle::MISSING_CONST_FN,
    Allow,
    "private helper can be declared const fn"
}

declare_tool_lint! {
    /// Flags mutating caller-owned state parameters in declared pure contexts.
    pub tigerstyle::MUTATING_INPUT_IN_PURE,
    Warn,
    "mutating input capability in pure context"
}

declare_tool_lint! {
    /// Advises when unordered hash iteration flows into ordered output without sorting.
    pub tigerstyle::NONDETERMINISTIC_OUTPUT,
    Allow,
    "unordered hash iteration collected into ordered output"
}

declare_tool_lint! {
    /// Enforces direct-effect purity checks for functions marked as pure.
    pub tigerstyle::PURE_ATTR_CONTRACT,
    Warn,
    "marked pure function contains a direct impurity"
}

declare_tool_lint! {
    /// Flags core functions that return concrete resource or capability handles.
    pub tigerstyle::RESOURCE_RETURN_FROM_CORE,
    Warn,
    "core function returns concrete resource handle"
}

declare_tool_lint! {
    /// Flags direct thread/task spawning in configured core scopes.
    pub tigerstyle::SPAWN_IN_CORE,
    Warn,
    "task or thread spawn in configured core scope"
}

declare_tool_lint! {
    /// Enforces direct-effect allocation checks for functions marked with the
    /// zero-allocation marker. Flags direct heap-allocation constructors in the
    /// annotated body and guides maintainers toward stack-local construction or
    /// pre-allocated buffers. This lint checks direct source shapes only, not
    /// transitive whole-program non-allocation proof.
    pub tigerstyle::ZERO_ALLOC_ATTR_CONTRACT,
    Warn,
    "marked zero-alloc function contains direct allocation"
}

pub struct AmbientEnv;
pub struct AmbientRandom;
pub struct GlobalStateInCore {
    core_source_scopes: Vec<String>,
}
pub struct ImpureCallInCore {
    core_source_scopes: Vec<String>,
}
pub struct InteriorMutabilityInCore {
    core_source_scopes: Vec<String>,
}
pub struct LoggingInCore {
    core_source_scopes: Vec<String>,
}
pub struct MissingConstFn;
pub struct MutatingInputInPure {
    core_source_scopes: Vec<String>,
}
pub struct NondeterministicOutput;
pub struct PureAttrContract;
pub struct ResourceReturnFromCore {
    core_source_scopes: Vec<String>,
}
pub struct SpawnInCore {
    core_source_scopes: Vec<String>,
}
pub struct ZeroAllocAttrContract;

impl GlobalStateInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl ImpureCallInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl InteriorMutabilityInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl LoggingInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl MutatingInputInPure {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl ResourceReturnFromCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl SpawnInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl_lint_pass!(AmbientEnv => [AMBIENT_ENV]);
impl_lint_pass!(AmbientRandom => [AMBIENT_RANDOM]);
impl_lint_pass!(GlobalStateInCore => [GLOBAL_STATE_IN_CORE]);
impl_lint_pass!(ImpureCallInCore => [IMPURE_CALL_IN_CORE]);
impl_lint_pass!(InteriorMutabilityInCore => [INTERIOR_MUTABILITY_IN_CORE]);
impl_lint_pass!(LoggingInCore => [LOGGING_IN_CORE]);
impl_lint_pass!(MissingConstFn => [MISSING_CONST_FN]);
impl_lint_pass!(MutatingInputInPure => [MUTATING_INPUT_IN_PURE]);
impl_lint_pass!(NondeterministicOutput => [NONDETERMINISTIC_OUTPUT]);
impl_lint_pass!(PureAttrContract => [PURE_ATTR_CONTRACT]);
impl_lint_pass!(ResourceReturnFromCore => [RESOURCE_RETURN_FROM_CORE]);
impl_lint_pass!(SpawnInCore => [SPAWN_IN_CORE]);
impl_lint_pass!(ZeroAllocAttrContract => [ZERO_ALLOC_ATTR_CONTRACT]);

fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn contains_any(source: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| source.contains(pattern))
}

fn contains_env_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    ENV_CALLS.iter().copied().find(|call| {
        let std_pattern = format!("std::env::{call}(");
        let short_pattern = format!("env::{call}(");
        compact_source.contains(&std_pattern) || compact_source.contains(&short_pattern)
    })
}

fn classify_random_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    RANDOM_CALL_PATTERNS.iter().copied().find(|pattern| compact_source.contains(pattern))
}

fn classify_spawn_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    SPAWN_CALL_PATTERNS.iter().copied().find(|pattern| compact_source.contains(pattern))
}

fn classify_logging_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    LOGGING_CALL_PATTERNS.iter().copied().find(|pattern| compact_source.contains(pattern))
}

fn classify_effect(source: &str) -> Option<EffectClass> {
    let compact_source = compact(source);
    if contains_env_call(&compact_source).is_some() {
        return Some(EffectClass::Env);
    }
    if classify_random_call(&compact_source).is_some() {
        return Some(EffectClass::Random);
    }
    if classify_spawn_call(&compact_source).is_some() {
        return Some(EffectClass::Spawn);
    }
    if classify_logging_call(&compact_source).is_some() {
        return Some(EffectClass::Logging);
    }
    if contains_any(&compact_source, FILESYSTEM_CALL_PATTERNS) {
        return Some(EffectClass::Filesystem);
    }
    if contains_any(&compact_source, IO_CALL_PATTERNS) {
        return Some(EffectClass::Io);
    }
    if contains_any(&compact_source, NET_CALL_PATTERNS) {
        return Some(EffectClass::Net);
    }
    if contains_any(&compact_source, PROCESS_CALL_PATTERNS) {
        return Some(EffectClass::Process);
    }
    if contains_any(&compact_source, TIME_CALL_PATTERNS) {
        return Some(EffectClass::Time);
    }
    if contains_any(&compact_source, RUNTIME_CALL_PATTERNS) {
        return Some(EffectClass::Runtime);
    }
    None
}

fn contains_global_state(source: &str) -> bool {
    let compact_source = compact(source);
    if source.contains("static mut ") || compact_source.contains("thread_local!(") {
        return true;
    }
    source.contains("static ") && contains_any(&compact_source, GLOBAL_STATE_PATTERNS)
}

fn contains_interior_mutability(source: &str) -> bool {
    let compact_source = compact(source);
    contains_any(&compact_source, INTERIOR_MUTABILITY_PATTERNS)
}

fn mutating_param_kind(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    MUTATING_PARAM_PATTERNS
        .iter()
        .copied()
        .find(|pattern| source.contains(pattern) || compact_source.contains(pattern))
}

fn return_type_has_resource(source: &str) -> Option<&'static str> {
    if contains_any(source, RESOURCE_DESCRIPTION_ALLOW_PATTERNS) {
        return None;
    }
    RESOURCE_RETURN_PATTERNS.iter().copied().find(|pattern| source.contains(pattern))
}

fn source_has_panic(source: &str) -> bool {
    let compact_source = compact(source);
    ["panic!(", "unimplemented!(", "unreachable!(", "todo!("]
        .iter()
        .any(|pattern| compact_source.contains(pattern))
}

fn source_has_pure_marker(source: &str) -> bool {
    let compact_source = compact(source);
    compact_source.contains("#[tigerstyle::pure]")
        || compact_source.contains("#[octet::pure]")
        || compact_source.contains("#[pure]")
}

fn source_has_zero_alloc_marker(source: &str) -> bool {
    let compact_source = compact(source);
    compact_source.contains("#[tigerstyle::zero_alloc]")
        || compact_source.contains("#[octet::zero_alloc]")
        || compact_source.contains("#[zero_alloc]")
}

fn preceding_attrs_have_pure_marker(cx: &LateContext<'_>, span: Span) -> bool {
    let source_map = cx.sess().source_map();
    let Ok(lines) = source_map.span_to_lines(span) else {
        return false;
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return false;
    };

    for idx in (0..first_line_idx).rev() {
        let Some(line) = lines.file.get_line(idx) else {
            continue;
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            if source_has_pure_marker(trimmed) {
                return true;
            }
            continue;
        }
        break;
    }
    false
}

fn preceding_attrs_have_zero_alloc_marker(cx: &LateContext<'_>, span: Span) -> bool {
    let source_map = cx.sess().source_map();
    let Ok(lines) = source_map.span_to_lines(span) else {
        return false;
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return false;
    };

    for idx in (0..first_line_idx).rev() {
        let Some(line) = lines.file.get_line(idx) else {
            continue;
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            if source_has_zero_alloc_marker(trimmed) {
                return true;
            }
            continue;
        }
        break;
    }
    false
}

fn fn_source_is_declared_pure(cx: &LateContext<'_>, _def_id: rustc_hir::def_id::LocalDefId, span: Span) -> bool {
    preceding_attrs_have_pure_marker(cx, span)
        || cx.sess().source_map().span_to_snippet(span).is_ok_and(|source| source_has_pure_marker(&source))
}

fn fn_source_is_declared_zero_alloc(cx: &LateContext<'_>, _def_id: rustc_hir::def_id::LocalDefId, span: Span) -> bool {
    preceding_attrs_have_zero_alloc_marker(cx, span)
        || cx
            .sess()
            .source_map()
            .span_to_snippet(span)
            .is_ok_and(|source| source_has_zero_alloc_marker(&source))
}

fn source_is_in_core_scope(cx: &LateContext<'_>, span: Span, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let filename = cx.sess().source_map().span_to_filename(span);
    let path = filename.prefer_local_unconditionally().to_string();
    scopes.iter().any(|scope| path.contains(scope))
}

fn late_expr_should_skip(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
}

fn expr_is_direct_call(expr: &hir::Expr<'_>) -> bool {
    matches!(expr.kind, hir::ExprKind::Call(..))
}

fn span_snippet(cx: &LateContext<'_>, span: Span) -> Option<String> {
    cx.sess().source_map().span_to_snippet(span.source_callsite()).ok()
}

fn expr_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    span_snippet(cx, expr.span)
}

fn fn_snippet(cx: &LateContext<'_>, body: &hir::Body<'_>, span: Span) -> Option<String> {
    span_snippet(cx, span).or_else(|| span_snippet(cx, body.value.span))
}

fn return_type_snippet(cx: &LateContext<'_>, decl: &hir::FnDecl<'_>) -> Option<String> {
    if let hir::FnRetTy::Return(ret_ty) = &decl.output {
        return span_snippet(cx, ret_ty.span);
    }
    None
}

fn params_contain_mutating_input(cx: &LateContext<'_>, body: &hir::Body<'_>) -> Option<(Span, &'static str)> {
    body.params.iter().find_map(|param| {
        let source = span_snippet(cx, param.pat.span)?;
        mutating_param_kind(&source).map(|kind| (param.pat.span, kind))
    })
}

fn decl_inputs_contain_mutating_type(cx: &LateContext<'_>, decl: &hir::FnDecl<'_>) -> Option<(Span, &'static str)> {
    decl.inputs.iter().find_map(|input| {
        let source = span_snippet(cx, input.span)?;
        mutating_param_kind(&source).map(|kind| (input.span, kind))
    })
}

fn function_has_nondeterministic_output(source: &str) -> bool {
    let compact_source = compact(source);
    if !contains_any(&compact_source, &["HashMap", "HashSet"]) {
        return false;
    }
    if contains_any(&compact_source, &["BTreeMap", "BTreeSet", ".sort", "sort_by", "sort_unstable"]) {
        return false;
    }
    let iterates_hash = contains_any(&compact_source, &[".iter(", ".keys(", ".values(", ".into_iter("]);
    let ordered_output = contains_any(&compact_source, &["->Vec<", "collect::<Vec", "Vec::from_iter"]);
    iterates_hash && ordered_output
}

fn const_primitive_type(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Bool | ty::Char | ty::Int(..) | ty::Uint(..) | ty::Float(..))
}

// Accept only operations whose operands are primitive values and whose HIR
// contains no calls, trait dispatch, mutation, indexing, loops, or effects.
// The bool records a real computation so a bare constant/identity is not linted.
fn const_primitive_expr(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<bool> {
    let interesting = match &expr.kind {
        hir::ExprKind::Lit(_) => false,
        hir::ExprKind::Path(qpath) => {
            match cx.qpath_res(qpath, expr.hir_id) {
                hir::def::Res::Local(_) | hir::def::Res::Def(hir::def::DefKind::Const { .. }, _) => {}
                _ => return None,
            }
            false
        }
        hir::ExprKind::Binary(_, left, right) => {
            const_primitive_expr(cx, left)?;
            const_primitive_expr(cx, right)?;
            true
        }
        hir::ExprKind::Unary(hir::UnOp::Not | hir::UnOp::Neg, operand) => {
            const_primitive_expr(cx, operand)?;
            true
        }
        hir::ExprKind::If(condition, then_expr, Some(else_expr)) => {
            if !cx.typeck_results().expr_ty(condition).is_bool() {
                return None;
            }
            const_primitive_expr(cx, condition)?;
            const_primitive_expr(cx, then_expr)?;
            const_primitive_expr(cx, else_expr)?;
            true
        }
        hir::ExprKind::Match(scrutinee, arms, hir::MatchSource::Normal) => {
            const_primitive_expr(cx, scrutinee)?;
            for arm in *arms {
                if arm.guard.is_some() {
                    return None;
                }
                const_primitive_expr(cx, arm.body)?;
            }
            true
        }
        hir::ExprKind::Block(block, _) if matches!(block.rules, hir::BlockCheckMode::DefaultBlock) => {
            let mut interesting = false;
            for stmt in block.stmts {
                interesting |= match stmt.kind {
                    hir::StmtKind::Let(local)
                        if local.els.is_none()
                            && matches!(local.pat.kind, hir::PatKind::Binding(_, _, _, None) | hir::PatKind::Wild)
                            && const_primitive_type(cx.typeck_results().pat_ty(local.pat)) =>
                    {
                        const_primitive_expr(cx, local.init?)?
                    }
                    hir::StmtKind::Expr(expr) | hir::StmtKind::Semi(expr) => const_primitive_expr(cx, expr)?,
                    _ => return None,
                };
            }
            interesting | const_primitive_expr(cx, block.expr?)?
        }
        hir::ExprKind::DropTemps(inner) => const_primitive_expr(cx, inner)?,
        _ => return None,
    };
    const_primitive_type(cx.typeck_results().expr_ty(expr)).then_some(interesting)
}

fn function_is_const_candidate(
    cx: &LateContext<'_>,
    kind: FnKind<'_>,
    body: &hir::Body<'_>,
    span: Span,
    def_id: rustc_hir::def_id::LocalDefId,
) -> bool {
    let FnKind::ItemFn(_, generics, header) = kind else {
        // Methods, trait functions, and closures have different const rules.
        return false;
    };
    let hir::Node::Item(item) = cx.tcx.hir_node(cx.tcx.local_def_id_to_hir_id(def_id)) else {
        return false;
    };
    if span.from_expansion()
        || !item.vis_span.is_empty()
        || cx.tcx.visibility(def_id).is_public()
        || !generics.params.is_empty()
        || !matches!(header.constness, hir::Constness::NotConst)
        || header.is_async()
        || !matches!(header.safety, hir::HeaderSafety::Normal(hir::Safety::Safe))
        || !matches!(header.abi, rustc_abi::ExternAbi::Rust)
    {
        return false;
    }
    let sig = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    sig.inputs().iter().all(|input| const_primitive_type(*input))
        && const_primitive_type(sig.output())
        && const_primitive_expr(cx, body.value) == Some(true)
}

fn pure_contract_violation(
    function_source: &str,
    return_source: Option<&str>,
    mutating_input: bool,
) -> Option<PureViolation> {
    if let Some(effect) = classify_effect(function_source) {
        return Some(PureViolation::DirectEffect(effect));
    }
    if contains_global_state(function_source) {
        return Some(PureViolation::GlobalState);
    }
    if contains_interior_mutability(function_source) {
        return Some(PureViolation::InteriorMutability);
    }
    if mutating_input {
        return Some(PureViolation::MutatingInput);
    }
    if source_has_panic(function_source) {
        return Some(PureViolation::Panic);
    }
    if return_source.is_some_and(|source| return_type_has_resource(source).is_some()) {
        return Some(PureViolation::ResourceReturn);
    }
    None
}

fn emit(cx: &LateContext<'_>, lint: &'static rustc_lint::Lint, span: Span, message: String, lint_name: &'static str) {
    emit_tiger_style_lint(cx, lint, span, message, lint_name);
}

impl<'tcx> LateLintPass<'tcx> for AmbientEnv {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr) || late_expr_should_skip(cx, expr) {
            return;
        }
        let Some(source) = expr_snippet(cx, expr) else {
            return;
        };
        let Some(call) = contains_env_call(&source) else {
            return;
        };
        emit(
            cx,
            AMBIENT_ENV,
            expr.span.source_callsite(),
            format!("direct `std::env::{call}` access reads or mutates ambient process state"),
            AMBIENT_ENV_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for AmbientRandom {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr) || late_expr_should_skip(cx, expr) {
            return;
        }
        let Some(source) = expr_snippet(cx, expr) else {
            return;
        };
        let Some(pattern) = classify_random_call(&source) else {
            return;
        };
        emit(
            cx,
            AMBIENT_RANDOM,
            expr.span.source_callsite(),
            format!("direct randomness source `{pattern}` makes core behavior non-deterministic"),
            AMBIENT_RANDOM_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for GlobalStateInCore {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if !source_is_in_core_scope(cx, item.span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span)
        {
            return;
        }
        let Some(source) = span_snippet(cx, item.span) else {
            return;
        };
        if !contains_global_state(&source) {
            return;
        }
        emit(
            cx,
            GLOBAL_STATE_IN_CORE,
            item.span,
            "core scope declares process-wide mutable or lazy state; thread explicit state through inputs instead"
                .to_owned(),
            GLOBAL_STATE_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for ImpureCallInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr)
            || !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || late_expr_should_skip(cx, expr)
        {
            return;
        }
        let Some(source) = expr_snippet(cx, expr) else {
            return;
        };
        let Some(effect) = classify_effect(&source) else {
            return;
        };
        emit(
            cx,
            IMPURE_CALL_IN_CORE,
            expr.span.source_callsite(),
            format!(
                "core scope performs direct {} effect; inject a port or move the call to the shell",
                effect.label()
            ),
            IMPURE_CALL_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for InteriorMutabilityInCore {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if !source_is_in_core_scope(cx, item.span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span)
        {
            return;
        }
        let Some(source) = span_snippet(cx, item.span) else {
            return;
        };
        if !contains_interior_mutability(&source) {
            return;
        }
        emit(
            cx,
            INTERIOR_MUTABILITY_IN_CORE,
            item.span,
            "core scope owns interior mutability; prefer explicit state transitions or shell-owned synchronization"
                .to_owned(),
            INTERIOR_MUTABILITY_IN_CORE_LINT,
        );
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr)
            || !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || late_expr_should_skip(cx, expr)
        {
            return;
        }
        let Some(source) = expr_snippet(cx, expr) else {
            return;
        };
        if !contains_interior_mutability(&source) {
            return;
        }
        emit(
            cx,
            INTERIOR_MUTABILITY_IN_CORE,
            expr.span.source_callsite(),
            "core scope constructs an interior-mutability primitive; keep synchronization in the shell".to_owned(),
            INTERIOR_MUTABILITY_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for LoggingInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes) || late_expr_should_skip(cx, expr) {
            return;
        }
        if expr.span.from_expansion() && !matches!(expr.kind, hir::ExprKind::Block(..)) {
            return;
        }
        let Some(source) = expr_snippet(cx, expr) else {
            return;
        };
        let Some(pattern) = classify_logging_call(&source) else {
            return;
        };
        emit(
            cx,
            LOGGING_IN_CORE,
            expr.span.source_callsite(),
            format!("core scope emits `{pattern}` directly; return events or diagnostics for the shell to log"),
            LOGGING_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for MissingConstFn {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }
        if !function_is_const_candidate(cx, kind, body, span, def_id) {
            return;
        }
        emit(
            cx,
            MISSING_CONST_FN,
            span,
            "private primitive helper appears const-compatible; consider declaring it `const fn`".to_owned(),
            MISSING_CONST_FN_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for MutatingInputInPure {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }
        let declared_pure = fn_source_is_declared_pure(cx, def_id, span);
        let in_core_scope = source_is_in_core_scope(cx, span, &self.core_source_scopes);
        if !declared_pure && !in_core_scope {
            return;
        }
        let Some((report_span, kind)) =
            decl_inputs_contain_mutating_type(cx, decl).or_else(|| params_contain_mutating_input(cx, body))
        else {
            return;
        };
        emit(
            cx,
            MUTATING_INPUT_IN_PURE,
            report_span,
            format!(
                "pure context accepts mutating input capability `{kind}`; return an explicit state transition instead"
            ),
            MUTATING_INPUT_IN_PURE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for NondeterministicOutput {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        _def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }
        let Some(source) = fn_snippet(cx, body, span) else {
            return;
        };
        if !function_has_nondeterministic_output(&source) {
            return;
        }
        emit(
            cx,
            NONDETERMINISTIC_OUTPUT,
            span,
            "hash-collection iteration flows into ordered output without a stable sort".to_owned(),
            NONDETERMINISTIC_OUTPUT_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for PureAttrContract {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
            || !fn_source_is_declared_pure(cx, def_id, span)
        {
            return;
        }
        let Some(source) = fn_snippet(cx, body, span) else {
            return;
        };
        let return_source = return_type_snippet(cx, decl);
        let mutating_input = decl_inputs_contain_mutating_type(cx, decl).is_some();
        let Some(violation) = pure_contract_violation(&source, return_source.as_deref(), mutating_input) else {
            return;
        };
        emit(
            cx,
            PURE_ATTR_CONTRACT,
            span,
            format!(
                "marked pure function contains direct {} impurity; this lint checks direct effects only, not transitive proof",
                violation.label()
            ),
            PURE_ATTR_CONTRACT_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for ZeroAllocAttrContract {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
            || !fn_source_is_declared_zero_alloc(cx, def_id, span)
        {
            return;
        }
        check_marked_zero_alloc_body(cx, body, span);
    }
}

fn check_marked_zero_alloc_body<'tcx>(cx: &LateContext<'tcx>, body: &'tcx hir::Body<'tcx>, fn_span: Span) {
    // Walk all expressions in the function body
    let mut finder = AllocExprFinder { cx, fn_span };
    finder.visit_expr(body.value);
}

#[allow(dead_code)]
struct AllocExprFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    fn_span: Span,
}

impl<'tcx> rustc_hir::intravisit::Visitor<'tcx> for AllocExprFinder<'_, 'tcx> {
    type NestedFilter = All;

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if let hir::ExprKind::Closure(_) = expr.kind {
            // Skip closures - they're separate functions
            return;
        }
        if let Some(type_name) = allocation_in_loop::expr_is_allocation(self.cx, expr) {
            let message = format!(
                "marked zero-alloc function constructs `{type_name}` which allocates on the heap; this lint checks direct allocation sites only, not transitive proof"
            );
            emit(self.cx, ZERO_ALLOC_ATTR_CONTRACT, expr.span, message, ZERO_ALLOC_ATTR_CONTRACT_LINT);
        }
        rustc_hir::intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for ResourceReturnFromCore {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        _def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if !source_is_in_core_scope(cx, span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
        {
            return;
        }
        let Some(return_source) = return_type_snippet(cx, decl) else {
            return;
        };
        let Some(resource) = return_type_has_resource(&return_source) else {
            return;
        };
        emit(
            cx,
            RESOURCE_RETURN_FROM_CORE,
            span,
            format!(
                "core function returns concrete resource `{resource}`; return a command description or domain value instead"
            ),
            RESOURCE_RETURN_FROM_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for SpawnInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr)
            || !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || late_expr_should_skip(cx, expr)
        {
            return;
        }
        let Some(source) = expr_snippet(cx, expr) else {
            return;
        };
        let Some(pattern) = classify_spawn_call(&source) else {
            return;
        };
        emit(
            cx,
            SPAWN_IN_CORE,
            expr.span.source_callsite(),
            format!("core scope starts work with `{pattern}`; return a command or use an injected executor port"),
            SPAWN_IN_CORE_LINT,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::EffectClass;
    use super::PureViolation;
    use super::classify_effect;
    use super::contains_env_call;
    use super::function_has_nondeterministic_output;
    use super::pure_contract_violation;
    use super::return_type_has_resource;

    #[test]
    fn env_classifier_accepts_direct_calls_and_rejects_explicit_config() {
        assert_eq!(contains_env_call("std::env::var(\"HOME\")"), Some("var"));
        assert_eq!(contains_env_call("config.home_dir.clone()"), None);
    }

    #[test]
    fn effect_classifier_keeps_shell_port_calls_out_of_scope() {
        assert_eq!(classify_effect("std::fs::read_to_string(path)"), Some(EffectClass::Filesystem));
        assert_eq!(classify_effect("ports.read_file(path)"), None);
    }

    #[test]
    fn resource_return_classifier_allows_descriptions() {
        assert_eq!(return_type_has_resource("std::fs::File"), Some("File"));
        assert_eq!(return_type_has_resource("CommandSpec"), None);
    }

    #[test]
    fn nondeterministic_output_requires_hash_iteration_without_sort() {
        assert!(function_has_nondeterministic_output(
            "fn names(map: HashMap<String, u8>) -> Vec<String> { map.keys().cloned().collect::<Vec<_>>() }"
        ));
        assert!(!function_has_nondeterministic_output(
            "fn names(map: HashMap<String, u8>) -> Vec<String> { let mut out = map.keys().cloned().collect::<Vec<_>>(); out.sort(); out }"
        ));
    }

    #[test]
    fn pure_contract_reports_direct_violation_and_accepts_clean_logic() {
        assert_eq!(
            pure_contract_violation("#[tigerstyle::pure] fn read() { std::fs::read(\"x\"); }", None, false),
            Some(PureViolation::DirectEffect(EffectClass::Filesystem))
        );
        assert_eq!(
            pure_contract_violation(
                "#[tigerstyle::pure] fn add(left: u32, right: u32) -> u32 { left + right }",
                None,
                false
            ),
            None
        );
    }
}
