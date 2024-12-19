using System;

class Program {
    static void Main() {
        for (int codeUnit = 0; codeUnit <= 0xFFFF; codeUnit++) {
            char c = (char)codeUnit;
            string both = $"{c}x{c}";
            string trimmed = both.Trim();
            if (trimmed != both) {
                string direction;
                if (trimmed == "x") {
                    direction = "on both sides";
                } else if (trimmed == $"x{c}") {
                    direction = "only on the left";
                } else if (trimmed == $"{c}x") {
                    direction = "only on the right";
                } else {
                    direction = "BUG";
                }
                Console.WriteLine($"U+{codeUnit:X4} is stripped {direction}");
            }
        }
    }
}
