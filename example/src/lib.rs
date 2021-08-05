use ion_shell::{builtins::Status, types::Str, Shell};

unsafe extern fn example_plugin_fn(args: &[Str], shell: &mut Shell) -> Status {
    println!("it worked");
    Status::SUCCESS
}