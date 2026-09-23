# Sandbox
You can emulate in-game battles and gameplay using the `Sandbox` page.

## Lineup
You can create and manage many lineups using the list on the right, with create, delete, and rename capabilities.

### Unit
You can add Units by using the cat list on the left, through either click or drag. The Bench is a store detached from the battle where you can put units you often swap between. Cats will show their level, cost, talent orbs, and whether they have any talents, as well as if the stages current restriction affects them.

To change a unit, double-click on their icon. Any changes made under this page will apply to the Unit in your lineup. Under "Talents" there is also a custom Orbs section that lets you set talent orbs for your Units.

Changes to a Unit are remembered, and the next time you select that unit under another line-up, its status will be restored. For mods, this behavior persists for the last 3 enabled mods only.

To remove a unit from a line-up, right click on them and the context menu will give you an option to do so, or you can drag the unit to any space outside the "Lineup" area.

### Combos
You can add a cat combo to the lineup by double-clicking it. You can also search cat combo's by attributes. Any cat combos that a stage restricts will be highlighted red, and active cat combos will be highlighted green.

## Stage
Mirror of the `Stages` page and determines what stage you lead into when you click `Play`.

## Config
Manage game settings using dedicated sections on a left sidebar.

- **Settings:** In-game battle settings alongside a meta `Device` setting that dictates HUD layout.
- **Treasure:** Treasure completion percentages for all 9 chapters, defaults to 100%.
- **Tech:** The tech upgrades you find alongside the Normal cats, as well as Cat Gods behavior and a meta `Level Cap` for the Aku Altar.
- **Base:** Cat Base Cannon, Style, and Foundation types as well as levels.
- **Items:** Which battle items you have equipped, note that some stages block some battle items.

## Replay
Save gameplay to lightweight `bcv` format, or export to other video formats.

### Body
- **Name:** An input field that changes and displays the name of the replay.
- **Meta:** Lists non-game details about the session.
- **Game:** Lists details that pertain to the game specifically.
- **Lineup:** The lineup that the replay uses, double click units for read-only information.
- **Manage:** Allows you to save the replay to disk and export it to specific formats.

### Behavior
Replays are saved automatically, which may increase load times. If you wish to opt-out of Replays and reclaim your load times, you can turn Replays off using `Settings > Sandbox > Replay > Disable Replays`.

Only your last gameplay session is saved. Said session is saved in a temporary folder, and is overwritten by the next session. To save it to a permanent file, you can click on the `Latest Battle` button and click `Save to Disk` under `Manage`.

When on this tab, the "Play" button instead becomes "Watch" which allows you to watch the replay. Exit the replay by clicking and holding the "Pause" keybind (default `Esc`).

Every file that the game loads is saved in the `.bcv` file. This means any Replay from any mod can be shared to other users that don't have that mod installed. If a later game update breaks the Replay, you can make Replays use your on-disk files instead by changing `Settings > Sandbox > Replay > Replay Source` to `VFS`.

Exporting to video formats requires the FFMPEG Addon, which you can install under `Settings > Addons > FFMPEG`.

## Keybinds
Battle can be controlled using Keybinds defined under `Settings > Sandbox > Keybinds`. There are even some Keybind exclusive behaviors, such as quickly restarting or ending a battle.