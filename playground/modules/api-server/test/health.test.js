import test from 'node:test';
import assert from 'node:assert/strict';

function sum(a, b) {
  return a + b;
}

test('sum adds operands', () => {
  assert.equal(sum(2, 3), 5);
});
