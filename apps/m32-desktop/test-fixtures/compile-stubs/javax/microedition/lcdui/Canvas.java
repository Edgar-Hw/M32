package javax.microedition.lcdui;
public abstract class Canvas extends Displayable {
    protected Canvas() { super(); }
    protected abstract void paint(Graphics graphics);
    protected void keyPressed(int keyCode) {}
    protected void keyReleased(int keyCode) {}
    protected void keyRepeated(int keyCode) {}
    public void repaint() {}
}
