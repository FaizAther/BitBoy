#!/usr/bin/env python3
"""Focused unit tests for the single SoftGBC play-agent harness."""

from __future__ import annotations

import json
import random
import tempfile
import unittest
from pathlib import Path

from play_agent import (
    BoardFitness,
    default_play_weights,
    generation_has_valid_bot,
    load_genome,
    pick_race_move,
    race_rank_key,
)


class PlayAgentRaceTests(unittest.TestCase):
    def test_old_genome_is_upgraded_with_play_weights(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "genome.json"
            path.write_text(
                json.dumps(
                    {
                        "weights": [1.0] * 7,
                        "fitness": 3,
                        "generation": 4,
                    }
                ),
                encoding="utf-8",
            )
            genome = load_genome(path)
            self.assertEqual(genome["play_weights"], default_play_weights())
            self.assertEqual(genome["best_score"], 0)

    def test_board_fitness_rewards_survival_and_estimated_clear(self):
        tracker = BoardFitness()
        tracker.observe([4] * 10, holes=0, bumpiness=0)
        before = tracker.score
        tracker.observe([3] * 10, holes=0, bumpiness=0)
        self.assertGreater(tracker.score, before)
        self.assertGreaterEqual(tracker.estimated_lines, 1)
        self.assertEqual(tracker.live_snapshots, 2)

    def test_board_fitness_penalizes_holes_and_height(self):
        clean = BoardFitness()
        dirty = BoardFitness()
        clean.observe([2] * 10, holes=0, bumpiness=0)
        dirty.observe([10] * 10, holes=8, bumpiness=9)
        self.assertGreater(clean.score, dirty.score)

    def test_live_score_breaks_stage_tie(self):
        low = {"stage": 5, "score": 100, "final": 5}
        high = {"stage": 5, "score": 900, "final": 5}
        self.assertGreater(race_rank_key(high), race_rank_key(low))
        menu_only = {"stage": 4, "score": 999999, "final": 4}
        self.assertGreater(race_rank_key(low), race_rank_key(menu_only))

    def test_failed_loads_do_not_form_a_generation(self):
        self.assertFalse(generation_has_valid_bot([{"error": "load_fail"}]))
        self.assertTrue(generation_has_valid_bot([{"error": "load_fail"}, {"error": None}]))

    def test_race_move_is_deterministic_for_seed(self):
        weights = default_play_weights()
        a = pick_race_move([0, 1, 2, 3, 4, 5, 4, 3, 2, 1], 7, weights, random.Random(99))
        b = pick_race_move([0, 1, 2, 3, 4, 5, 4, 3, 2, 1], 7, weights, random.Random(99))
        self.assertEqual(a, b)


if __name__ == "__main__":
    unittest.main()
