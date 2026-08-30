// M32 synthetic J2ME input source fixture.
// License: MIT, M32 contributors.

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;
import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;

public final class InputMidlet extends MIDlet {
    private final Canvas canvas = new Canvas() {
        private int lastKey = 0;

        protected void keyPressed(int keyCode) {
            lastKey = keyCode;
            repaint();
        }

        protected void paint(Graphics g) {
            g.setColor(0x000000);
            g.fillRect(0, 0, getWidth(), getHeight());
            g.setColor(0xFFFFFF);
            g.drawString("KEY:" + lastKey, 4, 4, Graphics.TOP | Graphics.LEFT);
        }
    };

    protected void startApp() {
        Display.getDisplay(this).setCurrent(canvas);
    }

    protected void pauseApp() {
    }

    protected void destroyApp(boolean unconditional) {
    }
}
