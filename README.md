# Julia Set Generator

Implements the challenge given at <https://www.reddit.com/r/dailyprogrammer/comments/4v5h3u/20160729_challenge_277_hard_trippy_julia_fractals/>.

## Problem Description

A Julia set is made by applying a function to the complex numbers repeatedly and keeping track of when the resulting numbers reach a threshold value. One number may take 200 iterations to achieve and absolute value over a certain threshold, value while an almost identical one might only take 10 iterations.

Here, we're interested in Julia sets because you can make pretty pictures with them if you map each complex input number to a pixel on the screen. The task today is to write a program that does all the math necessary for your computer to draw one of these beautiful pictures. In addition to making a buck from the band, you can also make a set of nice wallpapers for your desktop!

## Detailed Instructions

1. Pick your function

  Pick a function f which maps from a complex number z to another complex number. In our case we will use `f(z) = z^2 – 0.221 – 0.713 i`, because that makes a particularly pretty picture. To customize your own picture you can change the constant `– 0.221 – 0.713 i` to something else if you want. The threshold value for this function is `2`.

2. Make a set of complex numbers

  The only complex numbers which are interesting for the Julia set are the ones where both the real and the imaginary part is between `-1` and `1`. That's because, if the absolute value of an input number exceeds the threshold value, it will keep increasing or decreasing without bounds when you keep applying your function. So your program needs to keep a whole bunch of these small complex numbers in memory – one number for each pixel in your final image.

3. Apply f to that set of complex numbers iteratively

  Your program needs to check how many times you can apply the function f to each of the complex numbers above before its absolute value crosses the threshold value. So for each of your complex numbers, you get number of iterations, I.

4. Map the values of I to pixels in a picture

  You can do this in many ways, but an easier way, which I recommend, is that the real and imaginary parts of the complex numbers are the positions of the pixel on the X- and Y-axis, respectively, and I is the intensity of the pixel. You might want to set some cutoff to prevent specific pixels from iterating thousands of times.
