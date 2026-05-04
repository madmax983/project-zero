import os
import subprocess

def run_command(command):
    process = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate()
    return stdout.decode('utf-8'), stderr.decode('utf-8'), process.returncode

run_command('git add .')
commit_message = """feat(layer1): complete conveyor logistics system

Implements RED-GREEN-REFACTOR from spec 631:
- Added comprehensive tests for Inserter and overlap prevention
- Made ConveyorBelt an obstacle (blocks pathfinding)
- Added UndergroundConveyor and OverheadConveyor variants (non-obstacles)
- Implemented Inserter building and system to move items
- Improved conveyor_movement_system to prevent multiple items piling up
- Test coverage passes 85% requirement

All acceptance criteria met.

Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>"""

run_command(f'git commit -m "{commit_message}"')
