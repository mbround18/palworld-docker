use crate::runtime::{ContainerRuntime, ContainerSpec, ContainerStatus, RuntimeError};
#[path = "lifecycle/guard.rs"]
pub mod guard;

pub struct LifecycleManager<R: ContainerRuntime> {
    runtime: R,
}

impl<R: ContainerRuntime> LifecycleManager<R> {
    pub const STOP_TIMEOUT_SECONDS: u32 = 30;

    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }

    pub fn provision(&self, spec: &ContainerSpec) -> Result<ContainerStatus, RuntimeError> {
        self.runtime.ensure_image(&spec.image)?;
        let status = self.runtime.inspect(&spec.name)?;
        if status.exists {
            return Ok(status);
        }
        self.runtime.create(spec)?;
        self.runtime.inspect(&spec.name)
    }

    pub fn start(&self, name: &str) -> Result<ContainerStatus, RuntimeError> {
        let status = self.runtime.inspect(name)?;
        if status.running {
            return Ok(status);
        }
        self.runtime.start(name)?;
        self.runtime.inspect(name)
    }

    pub fn stop(&self, name: &str) -> Result<ContainerStatus, RuntimeError> {
        let status = self.runtime.inspect(name)?;
        if !status.exists || !status.running {
            return Ok(status);
        }
        self.runtime.stop(name, Self::STOP_TIMEOUT_SECONDS)?;
        self.runtime.inspect(name)
    }

    pub fn restart(&self, name: &str) -> Result<ContainerStatus, RuntimeError> {
        let status = self.runtime.inspect(name)?;
        if status.running {
            self.runtime.stop(name, Self::STOP_TIMEOUT_SECONDS)?;
        }
        self.runtime.start(name)?;
        self.runtime.inspect(name)
    }

    pub fn status(&self, name: &str) -> Result<ContainerStatus, RuntimeError> {
        self.runtime.inspect(name)
    }

    pub fn logs(&self, name: &str, follow: bool, tail: u32) -> Result<String, RuntimeError> {
        self.runtime.logs(name, follow, tail)
    }

    pub fn remove(&self, name: &str, force: bool) -> Result<(), RuntimeError> {
        let status = self.runtime.inspect(name)?;
        if !status.exists {
            return Ok(());
        }
        if status.running && !force {
            self.runtime.stop(name, Self::STOP_TIMEOUT_SECONDS)?;
        }
        self.runtime.remove(name, force)
    }

    pub fn update(&self, spec: &ContainerSpec) -> Result<ContainerStatus, RuntimeError> {
        self.runtime.ensure_image(&spec.image)?;
        let status = self.runtime.inspect(&spec.name)?;
        if !status.exists {
            self.runtime.create(spec)?;
            return self.runtime.inspect(&spec.name);
        }

        let was_running = status.running;
        if was_running {
            self.runtime.stop(&spec.name, Self::STOP_TIMEOUT_SECONDS)?;
        }
        self.runtime.remove(&spec.name, false)?;
        self.runtime.create(spec)?;
        if was_running {
            self.runtime.start(&spec.name)?;
        }
        self.runtime.inspect(&spec.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{HealthStatus, RuntimeError};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    #[derive(Default, Debug)]
    struct FakeRuntimeState {
        statuses: VecDeque<ContainerStatus>,
        calls: Vec<String>,
        created_specs: Vec<ContainerSpec>,
    }

    #[derive(Clone)]
    struct FakeRuntime {
        state: Rc<RefCell<FakeRuntimeState>>,
    }

    impl FakeRuntime {
        fn with_statuses(statuses: Vec<ContainerStatus>) -> (Self, Rc<RefCell<FakeRuntimeState>>) {
            let state = Rc::new(RefCell::new(FakeRuntimeState {
                statuses: VecDeque::from(statuses),
                calls: vec![],
                created_specs: vec![],
            }));
            (
                Self {
                    state: Rc::clone(&state),
                },
                state,
            )
        }
    }

    impl ContainerRuntime for FakeRuntime {
        fn ensure_image(&self, image: &str) -> Result<(), RuntimeError> {
            self.state
                .borrow_mut()
                .calls
                .push(format!("ensure_image:{image}"));
            Ok(())
        }

        fn inspect(&self, name: &str) -> Result<ContainerStatus, RuntimeError> {
            self.state.borrow_mut().calls.push(format!("inspect:{name}"));
            Ok(self
                .state
                .borrow_mut()
                .statuses
                .pop_front()
                .unwrap_or(ContainerStatus {
                    exists: true,
                    running: true,
                    health: HealthStatus::Healthy,
                }))
        }

        fn create(&self, spec: &ContainerSpec) -> Result<(), RuntimeError> {
            self.state
                .borrow_mut()
                .calls
                .push(format!("create:{}", spec.name));
            self.state.borrow_mut().created_specs.push(spec.clone());
            Ok(())
        }

        fn start(&self, name: &str) -> Result<(), RuntimeError> {
            self.state.borrow_mut().calls.push(format!("start:{name}"));
            Ok(())
        }

        fn stop(&self, name: &str, timeout_seconds: u32) -> Result<(), RuntimeError> {
            self.state
                .borrow_mut()
                .calls
                .push(format!("stop:{name}:{timeout_seconds}"));
            Ok(())
        }

        fn remove(&self, name: &str, force: bool) -> Result<(), RuntimeError> {
            self.state
                .borrow_mut()
                .calls
                .push(format!("remove:{name}:{force}"));
            Ok(())
        }

        fn logs(&self, _name: &str, _follow: bool, _tail: u32) -> Result<String, RuntimeError> {
            Ok("logs".to_owned())
        }
    }

    #[test]
    fn start_is_idempotent_when_already_running() {
        let (runtime, state) = FakeRuntime::with_statuses(vec![ContainerStatus {
            exists: true,
            running: true,
            health: HealthStatus::Healthy,
        }]);
        let lifecycle = LifecycleManager::new(runtime);

        let status = lifecycle.start("palworld").expect("start should succeed");

        assert!(status.running);
        assert_eq!(state.borrow().calls, vec!["inspect:palworld"]);
    }

    #[test]
    fn stop_is_idempotent_when_already_stopped() {
        let (runtime, state) = FakeRuntime::with_statuses(vec![ContainerStatus {
            exists: true,
            running: false,
            health: HealthStatus::None,
        }]);
        let lifecycle = LifecycleManager::new(runtime);

        let status = lifecycle.stop("palworld").expect("stop should succeed");

        assert!(!status.running);
        assert_eq!(state.borrow().calls, vec!["inspect:palworld"]);
    }

    #[test]
    fn restart_transitions_through_stop_then_start() {
        let (runtime, state) = FakeRuntime::with_statuses(vec![
            ContainerStatus {
                exists: true,
                running: true,
                health: HealthStatus::Healthy,
            },
            ContainerStatus {
                exists: true,
                running: true,
                health: HealthStatus::Healthy,
            },
        ]);
        let lifecycle = LifecycleManager::new(runtime);

        let status = lifecycle.restart("palworld").expect("restart should succeed");

        assert!(status.running);
        assert_eq!(
            state.borrow().calls,
            vec![
                "inspect:palworld".to_owned(),
                format!("stop:palworld:{}", LifecycleManager::<FakeRuntime>::STOP_TIMEOUT_SECONDS),
                "start:palworld".to_owned(),
                "inspect:palworld".to_owned()
            ]
        );
    }

    #[test]
    fn update_recreates_container_and_preserves_volume_spec() {
        let (runtime, state) = FakeRuntime::with_statuses(vec![
            ContainerStatus {
                exists: true,
                running: true,
                health: HealthStatus::Healthy,
            },
            ContainerStatus {
                exists: true,
                running: true,
                health: HealthStatus::Healthy,
            },
        ]);
        let lifecycle = LifecycleManager::new(runtime);
        let spec = ContainerSpec {
            name: "palworld".to_owned(),
            image: "mbround18/palworld-docker:latest".to_owned(),
            ports: vec!["8211:8211/udp".to_owned()],
            volumes: vec!["./data:/home/steam/palworld".to_owned()],
            env: vec![],
        };

        let status = lifecycle.update(&spec).expect("update should succeed");

        assert!(status.running);
        assert_eq!(state.borrow().created_specs.len(), 1);
        assert_eq!(
            state.borrow().created_specs[0].volumes,
            vec!["./data:/home/steam/palworld".to_owned()]
        );
    }

    #[test]
    fn remove_with_force_skips_stop_for_running_container() {
        let (runtime, state) = FakeRuntime::with_statuses(vec![ContainerStatus {
            exists: true,
            running: true,
            health: HealthStatus::Healthy,
        }]);
        let lifecycle = LifecycleManager::new(runtime);

        lifecycle
            .remove("palworld", true)
            .expect("force remove should succeed");

        assert_eq!(
            state.borrow().calls,
            vec!["inspect:palworld".to_owned(), "remove:palworld:true".to_owned()]
        );
    }
}
