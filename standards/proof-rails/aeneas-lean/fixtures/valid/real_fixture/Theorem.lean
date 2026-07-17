import RealFixture

/-- Named Octet-owned theorem over the generated Aeneas model. -/
theorem safe_add_matches_model : real_fixture.safe_add 1 2 = .ok 3 := by
  rfl
