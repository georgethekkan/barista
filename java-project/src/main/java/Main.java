import io.gth.App;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Main {
    private static Logger logger = LoggerFactory.getLogger(Main.class);
    public static void main() {
        System.out.println("Hello world");
        var emp = new Emp("sam", 30);
        var app = new App("My app");

        System.out.println(emp);
        System.out.println(app);
        logger.info("emp: {}, app: {}", emp, app);
    }
}
