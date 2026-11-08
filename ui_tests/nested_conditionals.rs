fn nested(value: u32) -> bool {
    if value > 0 {
        if value > 1 {
            if value > 2 {
                if value > 3 {
                    return true;
                }
            }
        }
    }

    false
}

fn main() {
    let _ = nested(4);
}

// Should NOT warn: a flat `else if` chain is one level of branching
fn flat_chain(value: u32) -> u32 {
    if value == 0 {
        0
    } else if value == 1 {
        1
    } else if value == 2 {
        2
    } else if value == 3 {
        3
    } else {
        4
    }
}

// Should warn: nesting inside an `else if` arm still counts from the chain level
fn nested_inside_chain(value: u32) -> bool {
    if value == 0 {
        false
    } else if value > 1 {
        if value > 2 {
            if value > 3 {
                if value > 4 {
                    return true;
                }
            }
        }
        false
    } else {
        true
    }
}

struct Router;

impl Router {
    // Should warn: inherent methods are checked like free functions
    fn nested_method(&self, value: u32) -> bool {
        if value > 0 {
            if value > 1 {
                if value > 2 {
                    if value > 3 {
                        return true;
                    }
                }
            }
        }

        false
    }
}

trait NestedDefault {
    // Should NOT warn: required trait methods have no body
    fn required_method(&self, value: u32) -> bool;

    // Should warn: provided trait methods have a body
    fn nested_default_method(&self, value: u32) -> bool {
        if value > 0 {
            if value > 1 {
                if value > 2 {
                    if value > 3 {
                        return true;
                    }
                }
            }
        }

        false
    }
}
