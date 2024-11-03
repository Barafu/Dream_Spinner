import webview
import sys

# Some control variables
ALLOW_QUIT = True  # Blocks quitting on mouse, for debug purposes


def is_release() -> bool:
    """Returns whether the app is running in release mode (aka inside PyInstaller)"""
    return getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS")


def determine_host_addr() -> str:
    """Returns the address to connnect the browser to, depending on whether the application is running from the bundle, or not."""
    if is_release():
        # in pyinstaller
        return "dist/index.html"
    else:
        # in development
        return "http://localhost:5173/"


class Connection:
    """Bundles functions to expose to JS side"""

    def __init__(self, app: "App"):
        self.app = app

    def log(self, message):
        print(message)

    def quit(self):
        if ALLOW_QUIT:
            self.app.stop()


def show_window(window):
    window.restore()


class App:
    def __init__(self):
        self.windows: list[webview.Window] = []
        self.connection = Connection(self)

    def create_windows(self):
        """Create one fullscreen window per monitor. Windows are initially hidden to avoid showing them before they are ready"""
        host_addr = determine_host_addr()
        for i, screen in enumerate(webview.screens):
            w = webview.create_window(
                f"DreamSpinner {i}",
                host_addr,
                screen=screen,
                fullscreen=True,
                js_api=self.connection,
                minimized=True,
                background_color="#000000",
            )
            w.events.loaded += show_window
            self.windows.append(w)

    def stop(self):
        """Stops the app, closing all its windows."""
        for w in self.windows:
            w.destroy()


if __name__ == "__main__":
    app = App()
    app.create_windows()
    webview.settings["OPEN_DEVTOOLS_IN_DEBUG"] = False
    allow_debug = not is_release()
    webview.start(debug=allow_debug)
