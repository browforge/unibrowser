import test from "node:test";
import assert from "node:assert/strict";

import {
  conflictingEnabledIds,
  isRelationshipCreateBlocked,
  missingRequiredIds,
  relationshipReason,
} from "../src/relationshipLogic.ts";

const rel = {
  requires: { "base-video": "Video handlers." },
  recommends: { "base-audio": "Audio mix." },
  optional: { "grammar-harper": "Spellcheck." },
  incompatible: { "legacy-video-flat": "Flat clip path." },
};

test("missing required blocks create", () => {
  assert.deepEqual(missingRequiredIds(rel, []), ["base-video"]);
  assert.equal(isRelationshipCreateBlocked(rel, [], []), true);
});

test("installed requires and no conflicts allow create", () => {
  assert.equal(isRelationshipCreateBlocked(rel, ["base-video"], ["base-video"]), false);
});

test("enabled incompatible mod blocks create", () => {
  assert.deepEqual(conflictingEnabledIds(rel, ["legacy-video-flat"]), [
    "legacy-video-flat",
  ]);
  assert.equal(
    isRelationshipCreateBlocked(rel, ["base-video"], ["base-video", "legacy-video-flat"]),
    true,
  );
});

test("relationship reason falls back", () => {
  assert.equal(relationshipReason("", "fallback"), "fallback");
  assert.equal(relationshipReason("custom", "fallback"), "custom");
});
