"""
rUGP COptimizedObs decoder — Pure Python implementation.

Based on reverse engineering of riooor.rpo (rUGP 5.80.20EC).
Reference functions are named in docstrings (e.g. sub_1000127C).

Usage:
    from obs_decoder import decode_obs_file, decode_obs_bytes
    pixels_bgra, width, height = decode_obs_file("COptimizedObs_xxx.bin")
"""

from .decoder import decode_obs_file, decode_obs_bytes, save_png
from .pipeline import ObsPipeline

__version__ = "2.0.0"
