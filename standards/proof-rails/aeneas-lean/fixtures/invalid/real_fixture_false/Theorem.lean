import RealFixture

/-- Negative fixture: the generated model cannot prove the false result. -/
theorem false_safe_add : real_fixture.safe_add 1 2 = .ok 4 := by
  rfl
