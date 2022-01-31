use crate::opt::InitArgs;

static INIT_SCRIPT: &str = include_str!("zabbrev-init.zsh");
static BIND_KEYS_SCRIPT: &str = include_str!("zabbrev-bindkey.zsh");

pub fn run(args: &InitArgs) {
    print!("{}", INIT_SCRIPT);

    if args.bind_keys {
        print!("{}", BIND_KEYS_SCRIPT);
    }
}
