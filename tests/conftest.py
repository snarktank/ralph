from __future__ import annotations

import sys
from pathlib import Path

# Allow tests to import project modules when pytest runs from tests/.
ROOT_DIR = Path(__file__).resolve().parents[1]
ROOT_STR = str(ROOT_DIR)
if ROOT_STR not in sys.path:
    sys.path.insert(0, ROOT_STR)
