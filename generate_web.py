#!/usr/bin/env python3
import os
import sys
import shutil
import subprocess
import argparse

def build_and_copy():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    wasm_dir = os.path.join(script_dir, 'wasm')
    pkg_dir = os.path.join(wasm_dir, 'pkg')
    generated_dir = os.path.join(script_dir, 'docs','generated')
    os.makedirs(generated_dir, exist_ok=True)

    print("Building WASM package...")
    result = subprocess.run(
        ["wasm-pack", "build", "--release", "--target", "web", "--no-typescript", "--no-pack"],
        cwd=wasm_dir,
    )
    if result.returncode != 0:
        print("Error: wasm-pack build failed")
        sys.exit(result.returncode)

    print("Copying package files to docs...")
    if not os.path.exists(pkg_dir):
        print(f"Error: pkg directory {pkg_dir} does not exist")
        sys.exit(1)

    for filename in os.listdir(pkg_dir):
        src_path = os.path.join(pkg_dir, filename)
        dest_path = os.path.join(generated_dir, filename)
        if os.path.isdir(src_path):
            if os.path.exists(dest_path):
                shutil.rmtree(dest_path)
            shutil.copytree(src_path, dest_path)
        else:
            shutil.copy2(src_path, dest_path)
    print("Build and copy completed successfully!")

def main():
    parser = argparse.ArgumentParser(description="Build and copy Lox WASM compiler.")
    parser.add_argument('--watch', action='store_true', help="Watch for changes and rebuild automatically")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    wasm_dir = os.path.join(script_dir, 'wasm')

    if args.watch:
        print("Starting watch mode using cargo-watch...")
        python_exe = sys.executable
        script_path = os.path.abspath(__file__)
        cmd = ["cargo", "watch", "-c", "-w", "src", "-s", f'"{python_exe}" "{script_path}"']
        subprocess.run(" ".join(cmd), cwd=wasm_dir, shell=True)
    else:
        build_and_copy()

if __name__ == '__main__':
    main()
