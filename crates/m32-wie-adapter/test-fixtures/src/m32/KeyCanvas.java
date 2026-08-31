package m32;

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;

public final class KeyCanvas extends Canvas {
    public KeyCanvas() {
        super();
    }

    protected void paint(Graphics graphics) {
        graphics.setColor(0x0E1114);
        graphics.fillRect(0, 0, 176, 220);

        graphics.setColor(0x3FA65A);
        graphics.fillRect(0, 0, 16, 16);
    }

    protected void keyPressed(int keyCode) {
        System.out.print("M32_KEY_PRESSED:");
        System.out.print(keyCode);
        System.out.print(";");
        System.out.flush();
    }

    protected void keyReleased(int keyCode) {
        System.out.print("M32_KEY_RELEASED:");
        System.out.print(keyCode);
        System.out.print(";");
        System.out.flush();
    }

    protected void keyRepeated(int keyCode) {
        System.out.print("M32_KEY_REPEATED:");
        System.out.print(keyCode);
        System.out.print(";");
        System.out.flush();
    }
}
