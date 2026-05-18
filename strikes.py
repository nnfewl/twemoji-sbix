"""Strike size presets for sbix font building."""

PRESETS = {
    "full": {
        "sizes": [20, 26, 32, 40, 48, 52, 64, 96, 160],
        "description": "All 9 strikes, matches Apple Color Emoji (~90 MB)",
    },
    "optimal": {
        "sizes": [32, 64, 128],
        "description": "3 strikes for terminal use on retina displays (~30 MB)",
    },
    "minimal": {
        "sizes": [64],
        "description": "Single strike, smallest possible (~10 MB)",
    },
}

DEFAULT_PRESET = "optimal"


def get_sizes(preset: str) -> list[int]:
    if preset not in PRESETS:
        raise ValueError(f"Unknown preset '{preset}'. Choose from: {', '.join(PRESETS)}")
    return PRESETS[preset]["sizes"]


def list_presets() -> str:
    lines = []
    for name, info in PRESETS.items():
        default = " (default)" if name == DEFAULT_PRESET else ""
        sizes_str = ", ".join(str(s) for s in info["sizes"])
        lines.append(f"  {name}{default}: [{sizes_str}] — {info['description']}")
    return "\n".join(lines)
