import Data.Char (isSpace, chr)
import Text.Printf (printf)

main :: IO ()
main = do
    let chars = [c | c <- [0..0x10FFFF], isSpace (chr c)]
    putStrLn "Characters matching Data.Char.isSpace:"
    mapM_ (\c -> printf "'%c' U+%04X\n" (chr c) c) chars
