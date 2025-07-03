# janky ass python code to extract the color pallette from a video of the real macos screensaver

import math
from collections import Counter

import cv2
import numpy

video_path = '/home/connorslade/Downloads/NEW Sequoia Macintosh Screen Saver HD [FULL VIDEO LOOP!!!] [pnjxehheT_I].webm'
brightness = 128

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
    return numpy.array(list([x] for x in colors), dtype=numpy.uint8)

def is_background(color):
    return color[0] >= brightness and color[1] >= brightness and color[2] >= brightness

def is_foreground(color):
    return color[0] < brightness and color[1] < brightness and color[2] < brightness

def add_background(list, row, invert = False):
    out = top_colors(row)
    top = Counter({color: count for color, count in out.items() if is_background(color) ^ invert})
    common = top.most_common(1)

    if len(common) > 0:
        list.append(common[0][0])

def add_foreground(list, image):
    out = top_colors(sum(image[int(height / 2):int(height / 2 + 1)]))
    top = Counter({color: count for color, count in out.items() if is_foreground(color)})
    common = top.most_common(1)

    if len(common) > 0:
        list.append(common[0][0])

cap = cv2.VideoCapture(video_path)
fps = cap.get(cv2.CAP_PROP_FPS)
height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))

top = []
bottom = []
foreground = []

frames = int(round(fps * 60))
for i in range(frames):
    print(f'{i}/{frames}')

    ret, frame = cap.read()
    if not ret:
        break

    add_background(top, frame[0])
    add_background(bottom, frame[height - 1])
    add_foreground(foreground, frame)

cap.release()

cv2.imwrite("top.png", colors_to_image(top))
cv2.imwrite("bottom.png", colors_to_image(bottom))
cv2.imwrite("foreground.png", colors_to_image(foreground))
