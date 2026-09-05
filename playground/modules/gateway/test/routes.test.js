import test from 'node:test';
import assert from 'node:assert/strict';

function routes() {
  return ['/health', '/'];
}

test('exposes health route', () => {
  assert.ok(routes().includes('/health'));
});
