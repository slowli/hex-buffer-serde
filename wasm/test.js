#!/usr/bin/env node

const { strict: assert } = require('assert');
const { reverse } = require('./pkg');

const value = {
  buffer: 'c0ffee',
  array_buffer: 'deadbeef',
  other_data: 'data',
};
const reversedValue = reverse(value);
assert.deepEqual(reversedValue, {
  buffer: 'eeffc0',
  array_buffer: 'efbeadde',
  other_data: 'data',
});
