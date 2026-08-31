package javax.microedition.midlet;
public abstract class MIDlet {
    protected MIDlet() {}
    protected abstract void startApp();
    protected void pauseApp() {}
    protected void destroyApp(boolean unconditional) {}
    public final void notifyDestroyed() {}
}
