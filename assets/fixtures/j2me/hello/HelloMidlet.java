// M32 synthetic J2ME source fixture.
// This file is intentionally source-only at T007.
// License: MIT, M32 contributors.

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Form;

public final class HelloMidlet extends MIDlet {
    private final Form form = new Form("M32 Fixture");

    public HelloMidlet() {
        form.append("HELLO-M32");
    }

    protected void startApp() {
        Display.getDisplay(this).setCurrent(form);
    }

    protected void pauseApp() {
    }

    protected void destroyApp(boolean unconditional) {
    }
}
