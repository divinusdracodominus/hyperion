x="hello"

if [ $x -eq "hello" ]; do
    echo "x == hello"
done

my_func(x, y) {
    echo "$x $y";
}

x=$(my_func "hello" "world")
echo $x