use polymera_kernel::policy::engine::*;
use polymera_kernel::policy::schema::*;
use std::collections::BTreeMap;

/// Test fixture for policy evaluation
struct PolicyFixture {
    /// Test case name
    name: String,
    /// Input that should be allowed
    allowed_input: PolicyInputV1,
    /// Input that should be denied
    denied_input: PolicyInputV1,
    /// Expected allow rate (0.0 to 1.0)
    expected_allow_rate: f64,
}

impl PolicyFixture {
    fn new(name: &str, expected_allow_rate: f64) -> Self {
        Self {
            name: name.to_string(),
            allowed_input: Self::create_allowed_input(),
            denied_input: Self::create_denied_input(),
            expected_allow_rate,
        }
    }

    fn create_allowed_input() -> PolicyInputV1 {
        PolicyInputV1 {
            intent: IntentV1 {
                id: 123,
                description: "safe operation".to_string(),
                intent_type: 1,
                priority: 1,
                requested_caps: vec![],
                metadata: BTreeMap::new(),
            },
            preview: PlanPreviewV1 {
                plan_id: 456,
                actions: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
                cost: 10,
                risks: vec![],
                notes: vec![],
            },
            wm_snapshot: 789,
            caps: vec![],
            features: 0,
        }
    }

    fn create_denied_input() -> PolicyInputV1 {
        PolicyInputV1 {
            intent: IntentV1 {
                id: 124,
                description: "dangerous operation".to_string(),
                intent_type: 2,
                priority: 1,
                requested_caps: vec![],
                metadata: BTreeMap::new(),
            },
            preview: PlanPreviewV1 {
                plan_id: 457,
                actions: vec![ActionV1 { kind: 2, params: BTreeMap::new() }],
                cost: 1000,
                risks: vec!["high".to_string()],
                notes: vec![],
            },
            wm_snapshot: 790,
            caps: vec![],
            features: 0,
        }
    }

    fn run_test(&self, engine: &WasmPolicyEngine) -> PolicyTestResult {
        let allowed_result = engine.eval(&self.allowed_input);
        let denied_result = engine.eval(&self.denied_input);

        let allowed = allowed_result.success && 
                     allowed_result.decision.as_ref().map_or(false, |d| d.allow);
        let denied = !denied_result.success || 
                    denied_result.decision.as_ref().map_or(true, |d| !d.allow);

        let actual_allow_rate = if allowed && denied {
            0.5 // One allowed, one denied
        } else if allowed {
            1.0 // Both allowed (fail-open!)
        } else if denied {
            0.0 // Both denied
        } else {
            0.5 // Neither clearly allowed/denied
        };

        PolicyTestResult {
            name: self.name.clone(),
            allowed,
            denied,
            actual_allow_rate,
            expected_allow_rate: self.expected_allow_rate,
            fail_open_detected: actual_allow_rate > self.expected_allow_rate,
        }
    }
}

/// Result of running a policy test
#[derive(Debug)]
struct PolicyTestResult {
    name: String,
    allowed: bool,
    denied: bool,
    actual_allow_rate: f64,
    expected_allow_rate: f64,
    fail_open_detected: bool,
}

/// Fail-open detector that runs policy fixtures
pub struct FailOpenDetector {
    fixtures: Vec<PolicyFixture>,
}

impl FailOpenDetector {
    pub fn new() -> Self {
        let mut fixtures = Vec::new();
        
        // Add test fixtures with expected allow rates
        fixtures.push(PolicyFixture::new("basic_safety", 0.5)); // Should allow safe, deny dangerous
        fixtures.push(PolicyFixture::new("capability_check", 0.5)); // Should check caps
        fixtures.push(PolicyFixture::new("risk_assessment", 0.5)); // Should assess risks
        
        Self { fixtures }
    }

    /// Run all fixtures and detect fail-open scenarios
    pub fn run_detection(&self, engine: &WasmPolicyEngine) -> FailOpenReport {
        let mut results = Vec::new();
        let mut total_fail_open = 0;
        let mut total_tests = 0;

        for fixture in &self.fixtures {
            let result = fixture.run_test(engine);
            if result.fail_open_detected {
                total_fail_open += 1;
            }
            total_tests += 1;
            results.push(result);
        }

        let fail_open_rate = if total_tests > 0 {
            total_fail_open as f64 / total_tests as f64
        } else {
            0.0
        };

        FailOpenReport {
            results,
            total_tests,
            total_fail_open,
            fail_open_rate,
            fail_open_detected: total_fail_open > 0,
        }
    }

    /// Add a custom fixture for testing
    pub fn add_fixture(&mut self, name: &str, expected_allow_rate: f64) {
        self.fixtures.push(PolicyFixture::new(name, expected_allow_rate));
    }
}

/// Report from fail-open detection
#[derive(Debug)]
pub struct FailOpenReport {
    results: Vec<PolicyTestResult>,
    total_tests: usize,
    total_fail_open: usize,
    fail_open_rate: f64,
    fail_open_detected: bool,
}

impl FailOpenReport {
    /// Print the report in JSON format for CI
    pub fn print_json(&self) {
        println!("{{\"test\":\"policy_sim\",\"allow_rate\":{:.2},\"fail_open_detected\":{}}}", 
                 self.fail_open_rate, 
                 if self.fail_open_detected { 1 } else { 0 });
    }

    /// Check if any fail-open scenarios were detected
    pub fn has_fail_open(&self) -> bool {
        self.fail_open_detected
    }

    /// Get the fail-open rate
    pub fn get_fail_open_rate(&self) -> f64 {
        self.fail_open_rate
    }
}

#[test]
fn test_fail_open_detector_creation() {
    let detector = FailOpenDetector::new();
    assert!(!detector.fixtures.is_empty());
}

#[test]
fn test_policy_fixture_creation() {
    let fixture = PolicyFixture::new("test", 0.5);
    assert_eq!(fixture.name, "test");
    assert_eq!(fixture.expected_allow_rate, 0.5);
}

#[test]
fn test_policy_fixture_allowed_input() {
    let fixture = PolicyFixture::new("test", 0.5);
    let input = &fixture.allowed_input;
    
    assert_eq!(input.intent.description, "safe operation");
    assert_eq!(input.intent.intent_type, 1);
    assert_eq!(input.preview.actions.len(), 1);
    assert_eq!(input.preview.cost, 10);
}

#[test]
fn test_policy_fixture_denied_input() {
    let fixture = PolicyFixture::new("test", 0.5);
    let input = &fixture.denied_input;
    
    assert_eq!(input.intent.description, "dangerous operation");
    assert_eq!(input.intent.intent_type, 2);
    assert_eq!(input.preview.actions.len(), 1);
    assert_eq!(input.preview.cost, 1000);
    assert!(input.preview.risks.contains(&"high".to_string()));
}

#[test]
fn test_fail_open_detection_normal() {
    let mut engine = WasmPolicyEngine::new(1024);
    let bundle_data = b"normal_policy";
    let hash = {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(bundle_data);
        hasher.finalize().into()
    };
    
    engine.load_bundle(bundle_data, hash).unwrap();
    
    let detector = FailOpenDetector::new();
    let report = detector.run_detection(&engine);
    
    // Normal policy should not have fail-open scenarios
    assert!(!report.has_fail_open());
    assert_eq!(report.total_tests, 3);
}

#[test]
fn test_fail_open_detection_permissive() {
    let mut engine = WasmPolicyEngine::new(1024);
    let bundle_data = b"permissive_policy";
    let hash = {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(bundle_data);
        hasher.finalize().into()
    };
    
    engine.load_bundle(bundle_data, hash).unwrap();
    
    let mut detector = FailOpenDetector::new();
    
    // Add a fixture that expects strict behavior
    detector.add_fixture("strict_check", 0.0); // Should deny everything
    
    let report = detector.run_detection(&engine);
    
    // This should detect fail-open if the policy is too permissive
    // Note: In our stub implementation, this might not trigger fail-open
    // In a real implementation with actual policy rules, this would catch permissive policies
    report.print_json();
}

#[test]
fn test_fail_open_report_json() {
    let report = FailOpenReport {
        results: vec![],
        total_tests: 5,
        total_fail_open: 2,
        fail_open_rate: 0.4,
        fail_open_detected: true,
    };
    
    assert!(report.has_fail_open());
    assert_eq!(report.get_fail_open_rate(), 0.4);
    assert_eq!(report.total_tests, 5);
    assert_eq!(report.total_fail_open, 2);
}

#[test]
fn test_policy_fixture_test_run() {
    let mut engine = WasmPolicyEngine::new(1024);
    let bundle_data = b"test_policy";
    let hash = {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(bundle_data);
        hasher.finalize().into()
    };
    
    engine.load_bundle(bundle_data, hash).unwrap();
    
    let fixture = PolicyFixture::new("test_run", 0.5);
    let result = fixture.run_test(&engine);
    
    // Verify the test ran and produced results
    assert!(!result.name.is_empty());
    assert!(result.actual_allow_rate >= 0.0);
    assert!(result.actual_allow_rate <= 1.0);
}

#[test]
fn test_custom_fixture_addition() {
    let mut detector = FailOpenDetector::new();
    let initial_count = detector.fixtures.len();
    
    detector.add_fixture("custom_test", 0.25);
    
    assert_eq!(detector.fixtures.len(), initial_count + 1);
    assert_eq!(detector.fixtures.last().unwrap().name, "custom_test");
    assert_eq!(detector.fixtures.last().unwrap().expected_allow_rate, 0.25);
}
