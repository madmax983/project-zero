#!/usr/bin/env python3
"""Interactive wrapper for the SCALE headless binary."""

import subprocess
import re
import time
import threading
import queue
from pathlib import Path

class ScaleGame:
    def __init__(self):
        project_root = Path(__file__).parent.parent
        self.binary = project_root / "target" / "debug" / "headless.exe"
        if not self.binary.exists():
            self.binary = project_root / "target" / "debug" / "headless"
        self.process = None
        self.output_queue = queue.Queue()

    def _reader(self):
        while self.process and self.process.poll() is None:
            try:
                line = self.process.stdout.readline()
                if line:
                    self.output_queue.put(line)
            except:
                break

    def start(self):
        self.process = subprocess.Popen(
            [str(self.binary)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True, bufsize=1
        )
        threading.Thread(target=self._reader, daemon=True).start()
        time.sleep(0.5)
        return self._collect_output()

    def _collect_output(self, timeout=0.3) -> str:
        lines = []
        deadline = time.time() + timeout
        while time.time() < deadline:
            try:
                line = self.output_queue.get(timeout=0.1)
                lines.append(line.rstrip())
                deadline = time.time() + 0.2
            except queue.Empty:
                pass
        return "\n".join(lines)

    def send(self, command: str, wait=0.5) -> str:
        self.process.stdin.write(command + "\n")
        self.process.stdin.flush()
        time.sleep(wait)
        return self._collect_output()

    def scan_area(self, cx, cy, radius=10):
        """Scan area and return tiles by type."""
        output = self.send(f"scan {cx} {cy} {radius}")
        tiles = {"grass": [], "dirt": [], "rock": [], "tree": [], "water": []}
        for line in output.split("\n"):
            match = re.search(r"TILE:\s*(\d+)\s+(\d+)\s+terrain=(\w+)", line)
            if match:
                x, y, terrain = int(match.group(1)), int(match.group(2)), match.group(3)
                if terrain in tiles:
                    tiles[terrain].append((x, y))
        return tiles

    def get_pop_positions(self) -> list[tuple[int, int]]:
        output = self.send("pops")
        positions = []
        for line in output.split("\n"):
            match = re.search(r"at \((\d+),(\d+)\)", line)
            if match:
                positions.append((int(match.group(1)), int(match.group(2))))
        return positions

    def quit(self):
        if self.process:
            try:
                self.process.stdin.write("quit\n")
                self.process.stdin.flush()
                self.process.wait(timeout=2)
            except:
                self.process.kill()


def manhattan_distance(p1, p2):
    return abs(p1[0] - p2[0]) + abs(p1[1] - p2[1])


def main():
    game = ScaleGame()
    print("=" * 60)
    print("SCALE Colony Simulation - Building Near Pops")
    print("=" * 60)
    print(game.start())

    pop_positions = game.get_pop_positions()
    print(f"\nPop positions: {pop_positions}")

    # Find centroid of pops to scan that area
    if pop_positions:
        cx = sum(p[0] for p in pop_positions) // len(pop_positions)
        cy = sum(p[1] for p in pop_positions) // len(pop_positions)
        print(f"Pop centroid: ({cx}, {cy})")
    else:
        cx, cy = 40, 25

    # Scan area around pops
    print(f"\n--- Scanning area around pops ---")
    tiles = game.scan_area(cx, cy, 15)
    print(f"Found near pops: {len(tiles['grass'])} grass, {len(tiles['rock'])} rock, {len(tiles['tree'])} tree")

    used_tiles = set()

    # Build farms on grass near pop centroid
    print("\n--- Building farms for food ---")
    for grass in sorted(tiles['grass'], key=lambda g: manhattan_distance((cx, cy), g))[:3]:
        x, y = grass
        if (x, y) not in used_tiles:
            result = game.send(f"build farm {x} {y}")
            if "Built" in result:
                print(f"Farm at ({x}, {y}) - dist {manhattan_distance((cx, cy), grass)} from centroid")
                used_tiles.add((x, y))

    # Build housing
    print("\n--- Building housing for rest ---")
    for grass in sorted(tiles['grass'], key=lambda g: manhattan_distance((cx, cy), g)):
        x, y = grass
        if (x, y) not in used_tiles:
            result = game.send(f"build housing {x} {y}")
            if "Built" in result:
                print(f"Housing at ({x}, {y})")
                used_tiles.add((x, y))
                break

    # Designate mining and chopping near pops
    print("\n--- Designating work near pops ---")
    for rock in sorted(tiles['rock'], key=lambda r: manhattan_distance((cx, cy), r))[:3]:
        x, y = rock
        result = game.send(f"mine {x} {y}")
        if "Designated" in result:
            print(f"Mine at ({x}, {y})")

    for tree in sorted(tiles['tree'], key=lambda t: manhattan_distance((cx, cy), t))[:3]:
        x, y = tree
        result = game.send(f"chop {x} {y}")
        if "Designated" in result:
            print(f"Chop at ({x}, {y})")

    print("\n--- Initial state ---")
    print(game.send("status"))
    print(game.send("buildings"))

    print("\n--- Running simulation ---")
    for batch in range(15):  # 1500 ticks
        game.send("tick 100", wait=0.3)
        tick = (batch + 1) * 100
        status = game.send("status")

        match = re.search(r"Tick (\d+)", status)
        tick_num = int(match.group(1)) if match else tick

        match = re.search(r"Population:\s*(\d+)", status)
        pop = int(match.group(1)) if match else 0

        match = re.search(r"Resources:\s*(\d+\.?\d*)\s*food.*?(\d+\.?\d*)\s*wood.*?(\d+\.?\d*)\s*stone", status)
        if match:
            food, wood, stone = float(match.group(1)), float(match.group(2)), float(match.group(3))
            print(f"Tick {tick_num}: {pop} pops | food={food:.1f} wood={wood:.1f} stone={stone:.1f}")

        # Stop early if all pops died
        if pop == 0:
            print("All pops died!")
            break

    print("\n--- Final state ---")
    print(game.send("status"))
    print(game.send("pops"))
    print(game.send("buildings"))
    print(game.send("designations"))

    game.quit()
    print("\nDone!")


if __name__ == "__main__":
    main()
