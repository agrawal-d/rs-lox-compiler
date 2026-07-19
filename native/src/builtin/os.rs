use std::env;
use std::process::{Command, exit};
use std::rc::Rc;
use std::cell::RefCell;

use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::{Value, value_as_string};

callable_struct!(Getenv, "getenv", 1, "getenv(name, [default])
Gets an environment variable. Returns default (or Nil) if not set.
Arguments:
  name: String environment variable name.
  default: (Optional) Fallback value if variable is not set.
Returns: String value or default/Nil.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let var_name = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s),
        _ => { set_global_error(interner, globals, "Expected string for environment variable name"); return Value::Nil; }
    };

    let default_val = if args.len() > 1 { args[1].clone() } else { Value::Nil };

    match env::var(var_name) {
        Ok(val) => Value::Str(interner.intern(&val)),
        Err(_) => default_val,
    }
});

callable_struct!(Setenv, "setenv", 2, "setenv(name, value)
Sets an environment variable.
Arguments:
  name: String environment variable name.
  value: String environment variable value.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let name_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string for environment variable name"); return Value::Nil; }
    };

    let val_str = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => value_as_string(&args[1], interner),
    };

    env::set_var(name_str, val_str);
    Value::Bool(true)
});

callable_struct!(Getenvs, "getenvs", 0, "getenvs()
Returns a Map containing all current environment variables.
Arguments: None.
Returns: Map of key-value environment pairs.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let mut map = rustc_hash::FxHashMap::default();
    for (k, v) in env::vars() {
        let k_val = Value::Str(interner.intern(&k));
        let v_val = Value::Str(interner.intern(&v));
        map.insert(k_val, v_val);
    }
    Value::Map(Rc::new(RefCell::new(map)))
});

callable_struct!(Exec, "exec", 1, "exec(cmd, [args_array])
Executes a system command and returns stdout, stderr, and exit_code in a Map.
Arguments:
  cmd: String command program to run.
  args_array: (Optional) Array of string arguments.
Returns: Map { \"stdout\": String, \"stderr\": String, \"exit_code\": Number }.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let cmd_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string command for os.exec"); return Value::Nil; }
    };

    let mut command = Command::new(&cmd_str);

    if args.len() > 1 {
        if let Value::Array(arr) = &args[1] {
            let borrow = arr.borrow();
            for arg_item in borrow.iter() {
                match arg_item {
                    Value::Str(s) | Value::Identifier(s) => {
                        command.arg(interner.lookup(s));
                    }
                    _ => {
                        command.arg(value_as_string(arg_item, interner));
                    }
                }
            }
        }
    }

    match command.output() {
        Ok(output) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1) as f64;

            let mut map = rustc_hash::FxHashMap::default();
            map.insert(Value::Str(interner.intern("stdout")), Value::Str(interner.intern(&stdout_str)));
            map.insert(Value::Str(interner.intern("stderr")), Value::Str(interner.intern(&stderr_str)));
            map.insert(Value::Str(interner.intern("exit_code")), Value::Number(exit_code));

            Value::Map(Rc::new(RefCell::new(map)))
        }
        Err(e) => {
            set_global_error(interner, globals, &format!("Failed to execute command '{}': {}", cmd_str, e));
            Value::Nil
        }
    }
});

callable_struct!(Args, "args", 0, "args()
Returns an Array of command-line arguments passed to the executable.
Arguments: None.
Returns: Array of strings.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let args_vec: Vec<Value> = env::args().map(|a| Value::Str(interner.intern(&a))).collect();
    Value::Array(Rc::new(RefCell::new(args_vec)))
});

callable_struct!(Cwd, "cwd", 0, "cwd()
Returns the current working directory path.
Arguments: None.
Returns: String path, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match env::current_dir() {
        Ok(p) => Value::Str(interner.intern(&p.to_string_lossy())),
        Err(e) => {
            set_global_error(interner, globals, &format!("Failed to get current directory: {}", e));
            Value::Nil
        }
    }
});

callable_struct!(Chdir, "chdir", 1, "chdir(path)
Changes current working directory.
Arguments:
  path: String target directory path.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let path_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s),
        _ => { set_global_error(interner, globals, "Expected string directory path for os.chdir"); return Value::Nil; }
    };

    match env::set_current_dir(path_str) {
        Ok(_) => Value::Bool(true),
        Err(e) => {
            set_global_error(interner, globals, &format!("Failed to change directory to '{}': {}", path_str, e));
            Value::Bool(false)
        }
    }
});

callable_struct!(Pid, "pid", 0, "pid()
Returns current process ID.
Arguments: None.
Returns: Number process ID.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    Value::Number(std::process::id() as f64)
});

callable_struct!(Platform, "platform", 0, "platform()
Returns target operating system name (\"windows\", \"linux\", \"macos\", etc.).
Arguments: None.
Returns: String platform name.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    Value::Str(interner.intern(std::env::consts::OS))
});

callable_struct!(Arch, "arch", 0, "arch()
Returns target CPU architecture (\"x86_64\", \"aarch64\", etc.).
Arguments: None.
Returns: String architecture name.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    Value::Str(interner.intern(std::env::consts::ARCH))
});

callable_struct!(Exit, "exit", 0, "exit([code])
Exits the process immediately with specified status code.
Arguments:
  code: (Optional) Number exit status code (default 0).
Returns: Never returns.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let code = if !args.is_empty() {
        match &args[0] {
            Value::Number(n) => *n as i32,
            _ => 0,
        }
    } else {
        0
    };
    exit(code);
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, Rc<dyn Callable>)] = &[
        ("getenv", Rc::new(Getenv)),
        ("setenv", Rc::new(Setenv)),
        ("getenvs", Rc::new(Getenvs)),
        ("exec", Rc::new(Exec)),
        ("args", Rc::new(Args)),
        ("cwd", Rc::new(Cwd)),
        ("chdir", Rc::new(Chdir)),
        ("pid", Rc::new(Pid)),
        ("platform", Rc::new(Platform)),
        ("arch", Rc::new(Arch)),
        ("exit", Rc::new(Exit)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
