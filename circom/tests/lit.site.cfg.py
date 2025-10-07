import os
import shutil
import subprocess
import tempfile
from pathlib import Path

# test_source_root: The root path where tests are located.
config.test_source_root = Path(__file__).parent

config.llvm_tools_dir = subprocess.check_output(
    ["llvm-config", "--bindir"], text=True
).rstrip()
config.available_features = {"circom"}

circom_path = config.test_source_root.parent / "target" / "debug" / "circom"
circom_path = circom_path.resolve() if circom_path.exists() else None
assert circom_path is not None, "circom not found on PATH"


config.circom_bin_dir = str(Path(circom_path).parent)
config.circom_src_dir = str(config.test_source_root.parent)

config.extra_suffixes = [".txt"]

import lit.llvm

lit.llvm.initialize(lit_config, config)


def _run_lit():
    # Let the main config do the real work.
    lit_config.load_config(config, config.test_source_root / "lit.cfg.py")


# test_exec_root: The root path where tests should be run.
override_test_exec_root = os.environ.get("CIRCOM_TEST_RUN", None)
if override_test_exec_root is None:
    with tempfile.TemporaryDirectory(prefix="circom_integration_test") as test_exec_root:
        config.test_exec_root = test_exec_root
        _run_lit()
else:
    assert Path(override_test_exec_root).is_dir()
    config.test_exec_root = override_test_exec_root
    _run_lit()
