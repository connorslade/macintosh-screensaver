# janky python code to extract the color pallette from a video of the real MacOs screensaver.
# The output of this needs a lot of manual work to be usable.

import math
from collections import Counter

import cv2
import numpy

video_path = '＂Macintosh＂ Dynamic Wallpaper from macOS 15 Sequoia [OYpfKUcANkw].webm'
height = 3606

def rgb_distance(a, b):
    return math.sqrt((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2)

def top_colors(pixels, threshold = 10):
    counts = Counter(map(lambda pixel: tuple(map(int, pixel)), pixels))
    out = {}

    for color, count in counts.items():
        found = False
        for key in out.keys():
            if rgb_distance(key, color) < threshold:
                out[key] += count
                found = True
                break

        if not found:
            out[color] = count

    return out

def colors_to_image(colors):
    return numpy.reshape(numpy.array(colors, dtype=numpy.uint8), (-1, height))

def add_color(list, row, foreground = False):
    top = Counter({color: count for color, count in top_colors(row).items() if (sum(color) / 3 > 100) ^ foreground})
    common = top.most_common(1)
    list.append(common[0][0] if len(common) > 0 else (0, 0, 0))

cap = cv2.VideoCapture(video_path)
fps = cap.get(cv2.CAP_PROP_FPS)
height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))

top = []
bottom = []
foreground = []

frames = height * 10
for i in range(frames):
    print(f'{i}/{frames}')

    ret, frame = cap.read()
    if not ret:
        break

    add_color(top, frame[0])
    add_color(bottom, frame[height - 1])

    center = sum(frame[int(height / 2):int(height / 2 + 1)])
    add_color(foreground, center, foreground = True)

cap.release()

cv2.imwrite("top.png", colors_to_image(top))
cv2.imwrite("bottom.png", colors_to_image(bottom))
cv2.imwrite("foreground.png", colors_to_image(foreground))
