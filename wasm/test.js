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

const invalidValue = {
  buffer: 'c0ffee',
  array_buffer: 'beef', // << invalid buffer length
  other_data: 'data',
};
assert.throws(() => reverse(invalidValue), {
  name: 'Error',
  message: /could not convert slice to array/i,
});
