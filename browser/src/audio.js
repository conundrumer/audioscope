function getMic (audio) {
  return new Promise((resolve, reject) => {
    const options = {
      video: false,
      audio: true
    }
    const onGetUserMedia = (stream) =>
      resolve(audio.createMediaStreamSource(stream))
    navigator.getUserMedia(options, onGetUserMedia, reject)
  })
}

export default function createAudio () {
  let N = 1024
  let samples = new Float32Array(N * 2)

  let context = new (window.AudioContext || window.webkitAudioContext)()

  let scriptNode = context.createScriptProcessor(N, 1, 1)
  scriptNode.onaudioprocess = (e) => {
    let inputBuffer = e.inputBuffer

    let inputData = inputBuffer.getChannelData(0)

    for (let i = 0; i < inputBuffer.length; i++) {
      samples[2 * i] = i / N
      samples[2 * i + 1] = inputData[i]
    }
  }

  getMic(context).then(input => {
    input.connect(scriptNode)
  }).catch((e) => {
    window.alert('audio setup failed')
    console.error(e)
  })

  scriptNode.connect(context.destination)

  return {
    getContext () {
      return context
    },
    getSamples () {
      return samples
    }
  }
}
