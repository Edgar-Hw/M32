package m32;

import javax.microedition.lcdui.Display;
import javax.microedition.midlet.MIDlet;

public final class PaintMidlet extends MIDlet {
    public PaintMidlet() {
        super();
    }

    protected void startApp() {
        Display.getDisplay(this).setCurrent(new PaintCanvas());
        System.out.println("M32_FIRST_FRAME_CANVAS_READY");
        System.out.flush();
    }
}
