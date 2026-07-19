use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use compiler::callable_struct;
use compiler::interner::Interner;
use compiler::native::{Callable, Globals, set_global_error};
use compiler::value::Value;

callable_struct!(ReadFile, "read_file", 1, "read_file(path)
Reads the entire text file into a string.
Arguments:
  path: String containing file path.
Returns: String content, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::read_to_string(path_str) {
                Ok(content) => Value::Str(interner.intern(&content)),
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to read file '{}': {}", path_str, e));
                    Value::Nil
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for read_file"); Value::Nil }
    }
});

callable_struct!(ReadFileBuf, "read_file_buf", 1, "read_file_buf(path)
Reads the entire binary file into a zero-copy Buffer.
Arguments:
  path: String containing file path.
Returns: Buffer object containing file bytes, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::read(path_str) {
                Ok(bytes) => Value::Buffer(Rc::new(RefCell::new(bytes))),
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to read binary file '{}': {}", path_str, e));
                    Value::Nil
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for read_file_buf"); Value::Nil }
    }
});

callable_struct!(WriteFile, "write_file", 2, "write_file(path, data)
Writes string or Buffer payload to a file (overwriting existing content).
Arguments:
  path: String containing file path.
  data: String or Buffer to write.
Returns: Bool (true on success, false on error).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    let path_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string path for write_file"); return Value::Nil; }
    };

    let res = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => {
            let content = interner.lookup(s);
            fs::write(&path_str, content)
        }
        Value::Buffer(buf) => {
            let bytes = buf.borrow();
            fs::write(&path_str, &*bytes)
        }
        _ => { set_global_error(interner, globals, "Expected string or buffer data for write_file"); return Value::Nil; }
    };

    match res {
        Ok(_) => Value::Bool(true),
        Err(e) => {
            set_global_error(interner, globals, &format!("Failed to write file '{}': {}", path_str, e));
            Value::Bool(false)
        }
    }
});

callable_struct!(AppendFile, "append_file", 2, "append_file(path, data)
Appends string or Buffer payload to a file.
Arguments:
  path: String containing file path.
  data: String or Buffer to append.
Returns: Bool (true on success, false on error).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    use std::io::Write;
    let path_str = match &args[0] {
        Value::Str(s) | Value::Identifier(s) => interner.lookup(s).to_string(),
        _ => { set_global_error(interner, globals, "Expected string path for append_file"); return Value::Nil; }
    };

    let file_res = fs::OpenOptions::new().create(true).append(true).open(&path_str);
    let mut file = match file_res {
        Ok(f) => f,
        Err(e) => {
            set_global_error(interner, globals, &format!("Failed to open file for append '{}': {}", path_str, e));
            return Value::Bool(false);
        }
    };

    let write_res = match &args[1] {
        Value::Str(s) | Value::Identifier(s) => file.write_all(interner.lookup(s).as_bytes()),
        Value::Buffer(buf) => file.write_all(&*buf.borrow()),
        _ => { set_global_error(interner, globals, "Expected string or buffer data for append_file"); return Value::Nil; }
    };

    match write_res {
        Ok(_) => Value::Bool(true),
        Err(e) => {
            set_global_error(interner, globals, &format!("Failed to append to file '{}': {}", path_str, e));
            Value::Bool(false)
        }
    }
});

callable_struct!(Exists, "exists", 1, "exists(path)
Checks if a file or directory exists.
Arguments:
  path: String path.
Returns: Bool.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => Value::Bool(Path::new(interner.lookup(s)).exists()),
        _ => { set_global_error(interner, globals, "Expected string path for exists"); Value::Nil }
    }
});

callable_struct!(IsFile, "is_file", 1, "is_file(path)
Checks if path points to a regular file.
Arguments:
  path: String path.
Returns: Bool.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => Value::Bool(Path::new(interner.lookup(s)).is_file()),
        _ => { set_global_error(interner, globals, "Expected string path for is_file"); Value::Nil }
    }
});

callable_struct!(IsDir, "is_dir", 1, "is_dir(path)
Checks if path points to a directory.
Arguments:
  path: String path.
Returns: Bool.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => Value::Bool(Path::new(interner.lookup(s)).is_dir()),
        _ => { set_global_error(interner, globals, "Expected string path for is_dir"); Value::Nil }
    }
});

callable_struct!(FileSize, "file_size", 1, "file_size(path)
Returns size of file in bytes.
Arguments:
  path: String path.
Returns: Number representing size, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::metadata(path_str) {
                Ok(meta) => Value::Number(meta.len() as f64),
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to get file size for '{}': {}", path_str, e));
                    Value::Nil
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for file_size"); Value::Nil }
    }
});

callable_struct!(RemoveFile, "remove_file", 1, "remove_file(path)
Deletes a file.
Arguments:
  path: String path.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::remove_file(path_str) {
                Ok(_) => Value::Bool(true),
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to remove file '{}': {}", path_str, e));
                    Value::Bool(false)
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for remove_file"); Value::Nil }
    }
});

callable_struct!(RemoveDir, "remove_dir", 1, "remove_dir(path)
Deletes a directory recursively.
Arguments:
  path: String path.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::remove_dir_all(path_str) {
                Ok(_) => Value::Bool(true),
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to remove dir '{}': {}", path_str, e));
                    Value::Bool(false)
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for remove_dir"); Value::Nil }
    }
});

callable_struct!(CreateDir, "create_dir", 1, "create_dir(path)
Creates directory recursively.
Arguments:
  path: String path.
Returns: Bool (true on success).",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::create_dir_all(path_str) {
                Ok(_) => Value::Bool(true),
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to create directory '{}': {}", path_str, e));
                    Value::Bool(false)
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for create_dir"); Value::Nil }
    }
});

callable_struct!(ReadDir, "read_dir", 1, "read_dir(path)
Lists all entry names in a directory.
Arguments:
  path: String path.
Returns: Array of strings, or Nil on error.",
interner: &mut Interner, globals: &mut Globals, args: &[Value], {
    match &args[0] {
        Value::Str(s) | Value::Identifier(s) => {
            let path_str = interner.lookup(s);
            match fs::read_dir(path_str) {
                Ok(entries) => {
                    let mut names = Vec::new();
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            names.push(Value::Str(interner.intern(name)));
                        }
                    }
                    Value::Array(Rc::new(RefCell::new(names)))
                }
                Err(e) => {
                    set_global_error(interner, globals, &format!("Failed to read directory '{}': {}", path_str, e));
                    Value::Nil
                }
            }
        }
        _ => { set_global_error(interner, globals, "Expected string path for read_dir"); Value::Nil }
    }
});

pub fn register(interner: &mut Interner, globals: &mut Globals, alias: &str) {
    let funcs: &[(&str, Rc<dyn Callable>)] = &[
        ("read_file", Rc::new(ReadFile)),
        ("read_file_buf", Rc::new(ReadFileBuf)),
        ("write_file", Rc::new(WriteFile)),
        ("append_file", Rc::new(AppendFile)),
        ("exists", Rc::new(Exists)),
        ("is_file", Rc::new(IsFile)),
        ("is_dir", Rc::new(IsDir)),
        ("file_size", Rc::new(FileSize)),
        ("remove_file", Rc::new(RemoveFile)),
        ("remove_dir", Rc::new(RemoveDir)),
        ("create_dir", Rc::new(CreateDir)),
        ("read_dir", Rc::new(ReadDir)),
    ];

    for (name, callable) in funcs {
        let full_name = format!("{}.{}", alias, name);
        let id = interner.intern(&full_name);
        globals.insert(id, Value::NativeFunction(callable.clone()));
    }
}
