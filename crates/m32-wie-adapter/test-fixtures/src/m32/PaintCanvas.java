package m32;

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;

public final class PaintCanvas extends Canvas {
    public PaintCanvas() {
        super();
    }

    protected void paint(Graphics graphics) {
        graphics.setColor(0x0E1114);
        graphics.fillRect(0, 0, 176, 220);

        graphics.setColor(0xD14A36);
        graphics.fillRect(0, 0, 16, 16);
    }
}
