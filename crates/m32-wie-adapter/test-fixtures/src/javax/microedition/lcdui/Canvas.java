package javax.microedition.lcdui;

public abstract class Canvas extends Displayable {
    protected Canvas() {
        super();
    }

    protected abstract void paint(Graphics graphics);
}
