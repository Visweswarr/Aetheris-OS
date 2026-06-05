//! Deterministic supervisor restart tests.

use polymera_supervisor::{ChildSpec, RestartPolicy, Strategy, Supervisor, SupervisorEvent, SupervisorHandle};
use tokio::time::{timeout, Duration};

fn exit_child(name: &str, code: i32, policy: RestartPolicy) -> ChildSpec {
    let args = if cfg!(windows) {
        vec!["/C".to_string(), format!("exit {}", code)]
    } else {
        vec!["-c".to_string(), format!("exit {}", code)]
    };
    ChildSpec::new(name, if cfg!(windows) { "cmd" } else { "sh" })
        .args(args)
        .restart_policy(policy)
}

async fn wait_for(
    handle: &mut SupervisorHandle,
    predicate: impl Fn(&SupervisorEvent) -> bool,
) -> SupervisorEvent {
    timeout(Duration::from_secs(3), async {
        loop {
            let event = handle.next_event().await.expect("supervisor event stream closed");
            if predicate(&event) {
                return event;
            }
        }
    })
    .await
    .expect("timed out waiting for supervisor event")
}

#[tokio::test(flavor = "multi_thread")]
async fn one_for_one_restarts_permanent_child() {
    let sup = Supervisor::new("test-root", Strategy::OneForOne)
        .max_restarts(10, 60)
        .child(exit_child("exit-loop", 0, RestartPolicy::Permanent));

    let mut handle = sup.start().expect("start");
    let event = wait_for(&mut handle, |event| matches!(event, SupervisorEvent::Restarted { name } if name == "exit-loop")).await;
    assert_eq!(event, SupervisorEvent::Restarted { name: "exit-loop".to_string() });
    handle.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn temporary_child_is_not_restarted() {
    let sup = Supervisor::new("test-root", Strategy::OneForOne)
        .child(exit_child("temporary", 0, RestartPolicy::Temporary));

    let mut handle = sup.start().expect("start");
    let event = wait_for(&mut handle, |event| matches!(event, SupervisorEvent::RestartSkipped { name, .. } if name == "temporary")).await;
    assert_eq!(event, SupervisorEvent::RestartSkipped {
        name: "temporary".to_string(),
        policy: RestartPolicy::Temporary,
    });
    handle.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn restart_rate_limit_stops_flapping_child() {
    let sup = Supervisor::new("test-root", Strategy::OneForOne)
        .max_restarts(0, 60)
        .child(exit_child("flap", 1, RestartPolicy::Permanent));

    let mut handle = sup.start().expect("start");
    let event = wait_for(&mut handle, |event| matches!(event, SupervisorEvent::RestartLimitExceeded { .. })).await;
    assert_eq!(event, SupervisorEvent::RestartLimitExceeded {
        restarts: 1,
        within_seconds: 60,
    });
    handle.shutdown().await;
}
