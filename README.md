# Spiralizer

Helps to create gifs like [this](https://www.reddit.com/r/gifs/comments/4xdfa9/timescape_halls_harbour_nova_scotia/).

Doesn't currently handle video frame extraction or creation.

## Usage
```
Spiralizer 0.1.0
Matt Ickstadt <mattico8@gmail.com>
Helps create a swirly timelapse gif

USAGE:
    spiralizer.exe <INPUT> <OUTPUT>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <INPUT>     A directory full of images to use
    <OUTPUT>    A directory to output the images to

Supported input formats: PNG JPG GIF ICO BMP
Output images will be saved in PNG format.
```

## Video Conversion With FFMpeg

Video to images:
ffmpeg -i video.mp4 -vf fps=1 frame_%04d.png
Adjust fps= to change how often frames are saved from the video.

Basic images to video:
ffmpeg -i frame%04d.png out.mp4

Recommended images to video settings:
ffmpeg -i frame%04d.png -framerate 60 -c:v libx264 -r 60 -pix_fmt yuv420p -profile slow -crf 19 out.mp4between images and video:

Video to images:
ffmpeg -i video.mp4 -vf fps=1 frame_%04d.png
Adjust fps= to change how often frames are saved from the video.

Basic images to video:
ffmpeg -i frame%04d.png out.mp4

Recommended images to video settings:
ffmpeg -i frame%04d.png -framerate 60 -c:v libx264 -r 60 -pix_fmt yuv420p profile slow -crf 19 out.mp4
