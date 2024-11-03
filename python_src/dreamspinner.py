import webview
import sys

# Some control variables
PACK_MODE = "DEV"
# PACK_MODE = "RELEASE"

ALLOW_QUIT = True

if PACK_MODE == "RELEASE":
    HOST_ADDR = "dist/index.html"
else:
    HOST_ADDR = "http://localhost:5173/"


class Connection:
    def __init__(self, app: "App"):
        self.app = app

    def log(self, message):
        print(message)

    def quit(self):
        if ALLOW_QUIT:
            self.app.stop()


class App:
    def __init__(self):
        self.windows: list[webview.Window] = []
        self.connection = Connection(self)

    def create_windows(self):
        for screen in webview.screens:
            w = webview.create_window(
                "Test",
                HOST_ADDR,
                screen=screen,
                resizable=True,
                width=1000,
                height=800,
                fullscreen=True,
                js_api=self.connection,
            )
            self.windows.append(w)

    def stop(self):
        for w in self.windows:
            w.destroy()


if __name__ == "__main__":
    app = App()
    app.create_windows()
    webview.settings["OPEN_DEVTOOLS_IN_DEBUG"] = False
    webview.start(debug=True)
