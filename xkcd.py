#!/usr/bin/env python3

from urllib import request
import json
import subprocess
from PIL import Image
import os
import random
import time
random.seed(time.time())


def remove_files():
    os.remove("xkcd0.png")
    os.remove("xkcd1.png")
    os.remove("xkcd2.png")
    os.remove("xkcd3.png")


def run(command):
    process = subprocess.Popen(command, stdout=subprocess.PIPE)
    output, error = process.communicate()
    if error:
        print(error)
    return str(output)


def get_xkcd_info(xkcd_url):
    data = request.urlopen(xkcd_url)
    return json.loads(data.read().decode())


def rescale_image():
    im = Image.open("xkcd0.png")
    width, height = im.size
    hscale = 1080 / height
    vscale = 1920 / width

    if hscale > vscale:
        run(["convert", "-resize", "1900x", "xkcd0.png", "xkcd1.png"])
    else:
        run(["convert", "-resize", "x1060", "xkcd0.png", "xkcd1.png"])


jsonObject = get_xkcd_info("http://xkcd.com/info.0.json")
# jsonObject = get_xkcd_info("http://xkcd.com/1732/info.0.json") # very tall
# jsonObject = get_xkcd_info("https://xkcd.com/859/info.0.json") # breaking mouseover

number = jsonObject['num']
# number = random.randint(1,number)

jsonObject = get_xkcd_info("http://xkcd.com/" + str(number) + "/info.0.json")

number = jsonObject['num']
title = jsonObject['title']
mouseover = jsonObject['alt']
imageurl = jsonObject['img']

request.urlretrieve(imageurl, "xkcd0.png")

rescale_image()

run(["convert", "xkcd1.png", "-gravity", "center", "-background", "white", "-extent", "1920x1080", "xkcd2.png"])
run(["convert", "xkcd2.png", "-gravity", "North", "-pointsize", "30", "-annotate", "+0+20", str(number) + ': ' + title,
     "xkcd3.png"])
run(["convert", "xkcd3.png", "-gravity", "South", "-pointsize", "30", "-annotate", "+0+20", mouseover, "xkcd.png"])

remove_files()

# vim: ts=4:et:sw=4:sts=4:noai
