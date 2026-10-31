"""F8 push-to-talk. Audio stays in memory; only recognized text leaves this worker."""
import ctypes
import json
import os
from pathlib import Path
import sys
import threading
import time
import queue
from contextlib import closing
from voice_input import open_microphone, mono_pcm

ROOT = Path(os.environ['ASSISTANT_VOICE_ROOT'])
sys.path.insert(0, str(ROOT / 'runtime/voice-chat-packages'))
sys.stdout.reconfigure(encoding='utf-8')


def emit(event, **fields):
    print(json.dumps(dict(event=event, **fields), ensure_ascii=False), flush=True)


def main():
    import sounddevice as sd
    from vosk import Model, KaldiRecognizer, SetLogLevel
    SetLogLevel(-1)
    # Kaldi's native Windows file APIs cannot open a model via a Unicode path.
    os.chdir(ROOT / 'runtime/asr/vosk-model-small-cn-0.22')
    model = Model('.')
    if '--probe' in sys.argv:
        emit('ready', devices=[d['name'] for d in sd.query_devices() if d['max_input_channels']])
        return
    stopped = threading.Event()
    enabled = threading.Event()
    enabled.set()

    def commands():
        for line in sys.stdin:
            if line.strip() == 'disable':
                enabled.clear()
            elif line.strip() == 'enable':
                enabled.set()
        stopped.set()

    threading.Thread(target=commands, daemon=True).start()
    key_down = lambda: bool(ctypes.windll.user32.GetAsyncKeyState(0x77) & 0x8000)
    was_down = key_down()
    emit('ready')
    while not stopped.wait(0.015):
        down = key_down()
        if down and not was_down and enabled.is_set():
            try:
                parts = []
                started = time.monotonic()
                # Open the microphone only while F8 is held.
                stream, chunks, rate, channels, device = open_microphone(sd)
                with closing(stream):
                    recognizer = KaldiRecognizer(model, rate)
                    emit('recording', device=device)
                    while key_down() and enabled.is_set() and not stopped.is_set() and time.monotonic() - started < 60:
                        try:
                            data, overflow = chunks.get(timeout=0.05)
                        except queue.Empty:
                            continue
                        if overflow:
                            raise RuntimeError('麦克风采集繁忙，请重试')
                        if recognizer.AcceptWaveform(mono_pcm(data, channels)):
                            parts.append(json.loads(recognizer.Result()).get('text', ''))
                if stopped.is_set():
                    break
                if not enabled.is_set():
                    emit('cancelled')
                else:
                    emit('processing')
                    parts.append(json.loads(recognizer.FinalResult()).get('text', ''))
                    text = ''.join(parts).replace(' ', '').strip()
                    emit('result', text=text[:2000])
            except Exception as error:
                emit('error', message='无法录音：' + str(error))
            # Require a fresh key press after timeout/error/cancellation.
            was_down = True
        else:
            was_down = down


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        emit('error', message='语音识别启动失败：' + str(error))
        sys.exit(1)
