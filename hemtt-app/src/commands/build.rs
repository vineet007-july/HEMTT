use crate::{flow::Stage, Command, Flow, HEMTTError, Project, Task};

pub struct Build {}
impl Command for Build {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("build")
            .version(*crate::VERSION)
            .about("Build the Project")
        // .args(&super::building_args())
    }

    fn run(&self, args: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
        let addons = crate::get_addons_from_args(args)?;
        let flow = Flow {
            tasks: {
                let mut tasks: Vec<Box<dyn Task>> = vec![
                    Box::new(crate::tasks::Clear {}),
                    Box::new(crate::tasks::NotEmpty {}),
                    Box::new(crate::tasks::ValidName {}),
                    Box::new(crate::tasks::ModTime {}),
                    Box::new(crate::tasks::Populate {}),
                    Box::new(crate::tasks::Prefix::new()),
                    Box::new(crate::tasks::Preprocess {}),
                    Box::new(crate::tasks::Rapify {}),
                    Box::new(crate::tasks::Pack {}),
                ];
                if args.is_present("force") {
                    tasks.push(Box::new(crate::tasks::Clean {}));
                }
                tasks
            },
        };
        flow.execute(addons, Stage::standard(), &p)?;
        Ok(())
    }
}
