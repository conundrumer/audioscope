export default function getMic () {
  // http://stackoverflow.com/questions/4723213/detect-http-or-https-then-force-https-in-javascript
  if (document.location.hostname !== 'localhost' && window.location.protocol !== 'https:') {
    window.location.href = 'https:' + window.location.href.substring(window.location.protocol.length)
  }

  navigator.getUserMedia = (navigator.getUserMedia || navigator.webkitGetUserMedia || navigator.mozGetUserMedia || navigator.msGetUserMedia)

  return new Promise((resolve, reject) => {
    const options = {
      video: false,
      audio: {
        optional: [
          {channelCount: 2}, // this doesn't work yet, only takes in summed mono
          {echoCancellation: false},
          {mozAutoGainControl: false},
          {mozNoiseSuppression: false},
          {googEchoCancellation: false},
          {googAutoGainControl: false},
          {googNoiseSuppression: false},
          {googHighpassFilter: false}
        ]
      }
    }
    const onGetUserMedia = (stream) => resolve(stream)
    navigator.getUserMedia(options, onGetUserMedia, reject)
  })
}
