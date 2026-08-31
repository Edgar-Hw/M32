package m32;

import javax.microedition.lcdui.Display;
import javax.microedition.midlet.MIDlet;

public final class KeyMidlet extends MIDlet {
    public KeyMidlet() {
        super();
    }

    protected void startApp() {
        Display.getDisplay(this).setCurrent(new KeyCanvas());
        System.out.print("M32_KEY_CANVAS_READY;");
        System.out.flush();
    }
}
