export default class MultiBuffer {
  constructor (N, numBuffers) {
    this.N = N
    this.numBuffers
    this.inputIndex = 0
    this.outputIndex = 0
    this.buffers = Array(numBuffers).fill().map(() => ({
      time: {
        re: new Float32Array(N),
        im: new Float32Array(N)
      }
    }))
  }
  getNextInputBuffer () {
    let buffer = this.buffers[this.inputIndex % this.numBuffers]
    if ((this.inputIndex - this.numBuffers) > this.outputIndex) {
      console.warning('dropped buffer!')
      this.outputIndex = this.inputIndex - this.numBuffers
    }
    this.inputIndex++
    return buffer
  }
  getNextOutputBuffer () {
    if (this.outputIndex >= this.inputIndex) {
      return null
    }
    let buffer = this.buffers[this.outputIndex % this.numBuffers]
    this.outputIndex++
    return buffer
  }
}
