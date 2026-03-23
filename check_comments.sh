#!/bin/bash
gh api repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/11/comments --jq '.[] | "\(.id) \(.user.login) \(.body[:80])"'
