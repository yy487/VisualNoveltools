# My Visual Novel Tools

[简体中文](README.md) | English

This is a collection of visual novel tools I maintain myself. It's a bit of a mixed bag, mainly covering:

- Unpacking and repacking
- Game image processing
- Script extraction and related utilities
- Other reverse engineering, format inspection, and patching tools

Progress varies from game to game. Some only have an unpacker, some have image or script tools, and others are still being investigated. It's normal for a game directory to be missing a particular kind of tool.

## Directory Structure

The general idea is one folder per game.

### Naming

- Tools are grouped by engine, platform, or developer. Game folders use the original game title or a commonly used translated title whenever possible.
- When the game title hasn't been confirmed, I keep the known format name or the existing category name.
- Sometimes I use the developer's name instead. It mostly depends on what comes to mind at the time.
- Games using the same engine are kept in separate game directories where possible.

### The `common` Directory

If an engine directory has a `common` folder, that usually means its basic formats have been worked out fairly well. Reusable tools and shared code go there.

Still, start with the tools in the specific game's directory. Those have usually been tested against that game, so their parameters, keys, encodings, and handling of unusual cases are more likely to match your version.

## Usage

1. Check the game and version first, then choose the tools from its directory.
2. Keep a copy of the original files before unpacking, repacking, or injecting text. Write the output to a new file or directory.
3. Different releases of the same game can use different formats. A tool running successfully doesn't mean its output will work with every release. For example, the Steam release of Aokana uses a Unity-adapted BGI setup, and some re-releases may switch engines altogether. It depends on the version.
4. If a tool can't handle your files, open an issue with the game name, version, filename, and error message so I can look into it.

## AI Translation Patches

If these tools help you make an AI translation patch, you're welcome to include a link to this repository when you release it. That helps other people find the tools and makes future fixes and improvements easier:

[https://github.com/yy487/vn-tool](https://github.com/yy487/vn-tool)
