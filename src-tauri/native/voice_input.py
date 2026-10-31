"""Explicit microphone selection, including Windows drivers without a default input."""
import queue


def input_candidates(sd):
    devices = sd.query_devices()
    try:
        default = sd.default.device[0]
    except Exception:
        default = -1
    candidates = []
    for index, device in enumerate(devices):
        name = device['name'].lower()
        if not device['max_input_channels']:
            continue
        # Never silently record speaker mix or a line input instead of speech.
        if any(word in name for word in ('stereo', 'mix', '立体声', '混音', 'line', '线路')):
            continue
        priority = 0 if index == default else 1 if any(word in name for word in ('mic', '麦克风')) else 2
        candidates.append((priority, index, device))
    return [(index, device) for _, index, device in sorted(candidates, key=lambda item: (item[0], item[1]))]


def open_microphone(sd):
    candidates = input_candidates(sd)
    if not candidates:
        raise RuntimeError('未找到可用麦克风，请连接麦克风并在 Windows 声音设置中启用，然后重启应用')
    for index, device in candidates:
        # Native rates/stereo are required by some Realtek and Bluetooth drivers.
        rates = list(dict.fromkeys([int(device['default_samplerate']), 16000, 48000, 44100]))
        channels = list(dict.fromkeys([min(2, device['max_input_channels']), 1]))
        for rate in rates:
            for channel_count in channels:
                stream = None
                try:
                    sd.check_input_settings(device=index, samplerate=rate, channels=channel_count, dtype='int16')
                    chunks = queue.Queue(maxsize=40)

                    def callback(data, frames, timing, status, target=chunks):
                        try:
                            target.put_nowait((bytes(data), bool(status)))
                        except queue.Full:
                            pass

                    # WDM-KS supports callback capture, not blocking stream.read().
                    stream = sd.RawInputStream(device=index, samplerate=rate, channels=channel_count,
                                               dtype='int16', blocksize=0, callback=callback)
                    stream.start()
                    return stream, chunks, rate, channel_count, device['name']
                except Exception:
                    if stream is not None:
                        stream.close()
    raise RuntimeError('麦克风无法打开，请检查设备连接、Windows 麦克风权限，以及是否被其他程序独占；启用后重启应用')


def mono_pcm(data, channels):
    if channels == 1:
        return data
    import audioop
    return audioop.tomono(data, 2, 0.5, 0.5)
