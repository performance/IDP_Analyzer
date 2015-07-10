# IDP_Analyzer
A simple tool to help learn Rust

# The format for IDP images:
1. All values are in LittleEndian
2. first u32 is always 0
3. second u32 is 0 for u16, 16 bit GrayScale images
4. second u32 is 2 for f32, floating point images
5. third u32 is number of columns
6. fourth u32 is number of rows
7. subsequent data is pixels in row major order, i.e, pixels of 0th row, followed by pixels of 1st row etc..

# This tools should be able to:
1. Read both types of IDP files
2. Perform simple operations like subtract one image from another.
3. Save an image in .IDP format described above
4. Manipulate individual pixels efficiently.
5. Calculate mean, median of whole frame, each row, each column
6. Count number of pixels less than a threshold 
7. Mask certain pixels as dead pixels and not include them in statistics
8. ... More to come..


Some of the code is copied from https://github.com/PistonDevelopers/image

I hope to eventually contribute to that repo too.

