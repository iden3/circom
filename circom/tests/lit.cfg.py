#!/usr/bin/env python3
# Modified version of
# https://raw.githubusercontent.com/llvm/llvm-project/main/mlir/examples/standalone/test/lit.cfg.py
# from LLVM, which is licensed under Apache 2.0 with LLVM Exceptions.

# -*- Python -*-

import os
import platform
import re
import subprocess
import tempfile

import lit.formats
import lit.util
from lit.llvm import llvm_config
from lit.llvm.subst import FindTool, ToolSubst

# Configuration file for the 'lit' test runner.

assert hasattr(config, "llvm_tools_dir")
assert hasattr(config, "circom_bin_dir")
assert hasattr(config, "extra_suffixes")
assert hasattr(config, "test_exec_root")
assert hasattr(config, "available_features")
# assert hasattr(config, 'test_source_root')

# name: The name of this test suite.
config.name = "circom"

config.test_format = lit.formats.ShTest(not llvm_config.use_lit_shell)

# suffixes: A list of file extensions to treat as test files.
config.suffixes = [".circom"]
config.suffixes.extend(config.extra_suffixes)

# test_source_root: The root path where tests are located.

# excludes: A list of directories to exclude from the testsuite. The 'Inputs'
# subdirectories contain auxiliary inputs for various tests in their parent
# directories.
config.excludes = ["Inputs", "CMakeLists.txt", "README.txt", "LICENSE.txt"]


# Set up environment variables

llvm_config.with_system_environment(["HOME", "INCLUDE", "LIB", "TMP", "TEMP"])

llvm_config.use_default_substitutions()
llvm_config.with_environment("PATH", config.circom_bin_dir, append_path=True)
llvm_config.with_environment("PATH", config.llvm_tools_dir, append_path=True)
llvm_config.with_environment("FILECHECK_OPTS", "--dump-input-filter=all")

tool_dirs = [config.circom_bin_dir, config.circom_src_dir]
tools = ["circom"]

# Set up variable substitutions
llvm_config.add_tool_substitutions(tools, tool_dirs)
config.substitutions.append(("%PATH%", config.environment["PATH"]))
