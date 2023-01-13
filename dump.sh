#!/bin/bash
find . -name '*.rs' -printf "%d %P\n" | sort -n | cut -c3- | xargs -i bash -c \
	'printf "\n\n/* -------------------------------- */\n/* %-32s */\n/* -------------------------------- */\n\n" "{}"; cat "{}"'
