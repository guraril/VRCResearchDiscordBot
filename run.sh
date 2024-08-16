#!/bin/bash
python3 -m venv celestnode_discord_bot_venv
./celestnode_discord_bot_venv/bin/python3 main.py
# clear virtual environment
rm -rf celestnode_discord_bot_venv
