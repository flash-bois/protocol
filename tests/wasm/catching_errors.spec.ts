import { Connection, PublicKey } from '@solana/web3.js'
import { assert } from 'chai'
import { describe } from 'mocha'
import { ret_error } from '../../pkg/protocol'

describe('catching errors in try catch', async () => {
  it('catches error', async () => {
    try {
      ret_error();
    } catch (error) {
      assert.equal(error, "Provided index is out of bounds")
    }
  })
})
