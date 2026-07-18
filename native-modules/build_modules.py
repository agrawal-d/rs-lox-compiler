#!/usr/bin/env python3
import os
import sys
import subprocess
import shutil
import platform

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
INCLUDE_DIR = os.path.join(ROOT, "include")
NATIVE_DIR = os.path.join(ROOT, "native-modules")
OUT_DIR = os.path.join(NATIVE_DIR, "build")

def find_compiler():
    if platform.system() == "Windows":
        # Check standard PATH
        for exe in ("g++.exe", "clang++.exe", "cl.exe"):
            path = shutil.which(exe)
            if path:
                return path, exe
        # If cl.exe isn't directly on path, check common locations or default to g++
        raise RuntimeError("No suitable C++ compiler (g++, clang++, cl) found on PATH. Please run in a Developer Command Prompt or install MinGW/MSVC.")
    else:
        for exe in ("g++", "clang++"):
            path = shutil.which(exe)
            if path:
                return path, exe
        raise RuntimeError("No suitable C++ compiler (g++, clang++) found on PATH.")

def compile_module(src_path):
    compiler_path, exe_name = find_compiler()
    name = os.path.splitext(os.path.basename(src_path))[0]
    
    is_windows = platform.system() == "Windows"
    ext = "dll" if is_windows else "so"
    out_file = os.path.join(OUT_DIR, f"{name}.{ext}")
    
    # Compile flags
    if exe_name == "cl.exe":
        cmd = [
            compiler_path,
            "/LD",
            "/O2",
            "/EHsc",
            "/std:c++17",
            f"/I{INCLUDE_DIR}",
            src_path,
            f"/Fe{out_file}"
        ]
    else:
        cmd = [
            compiler_path,
            "-shared",
            "-fPIC",
            "-O3",
            "-std=c++17",
            f"-I{INCLUDE_DIR}",
            src_path,
            "-o", out_file
        ]
        
    print("Compiling:", " ".join(cmd))
    subprocess.check_call(cmd)

def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    # Find all cpp files in NATIVE_DIR (excluding subdirectories or build folder)
    compiled = 0
    for fname in os.listdir(NATIVE_DIR):
        if fname.endswith(".cpp"):
            compile_module(os.path.join(NATIVE_DIR, fname))
            compiled += 1
    if compiled == 0:
        print("No .cpp files found to compile in native-modules/")
    else:
        print(f"Successfully compiled {compiled} module(s).")

if __name__ == "__main__":
    main()
