#!/usr/env python3

import os
import requests

stats = {}

for filename in os.listdir("translations/"):
    if not filename.endswith('.ts'):
        continue
    if not filename.startswith('harbour-whisperfish-'):
        print("WARN: " + filename + " does not follow format")
        continue
    lang = filename.split('-')[2].split('.')[0]
    print("processing language " + lang)
    stats[lang] = {
        'numerator': 0,  # unfinished count
        'denominator': 0,  # total count
    }

    with open("translations/" + filename, 'r') as f:
        for line in f:
            if "<translation" in line:
                stats[lang]['denominator'] += 1
                if "type=\"unfinished\"" in line:
                    stats[lang]['numerator'] += 1

unfinished = []
finished = []

numerator = 0
denominator = 0

for lang, stat in stats.items():
    print(lang + " has " + str(stat["numerator"]) + " unfinished translations")
    numerator += stat["numerator"]
    denominator += stat["denominator"]
    if stat["numerator"] > 0:
        unfinished.append(lang)
    else:
        finished.append(lang)

frac = int((denominator - numerator)/denominator * 100)
print(str(frac) + "% lines are translated")

color = "brightgreen"
if frac < 90:
    color = "orange"
if frac < 70:
    color = "red"
if frac < 50:
    color = "critical"

localized = "https://img.shields.io/badge/Localized-" + str(frac) + "%25-" + color
print(localized)

frac = len(finished) / (len(finished) + len(unfinished)) * 100
print(str(frac) + "% languages are translated")

color = "brightgreen"
if frac < 90:
    color = "orange"
if frac < 70:
    color = "red"
if frac < 50:
    color = "critical"

languages = "https://img.shields.io/badge/Languages-" + str(len(finished)) + "%2F" + str(len(finished)+len(unfinished)) + "-" + color
print(languages)

# Now upload to Gitlab

project_id = os.environ["CI_PROJECT_ID"]
token = os.environ["PRIVATE_TOKEN"]

languages_id = os.environ["TS_LANGUAGES_BADGE_ID"]
messages_id = os.environ["TS_MESSAGES_BADGE_ID"]

base_url = "https://gitlab.com/api/v4/projects/" + str(project_id) + "/badges/"

job_url = os.environ["CI_JOB_URL"]

response = requests.put(base_url + languages_id, headers={"PRIVATE-TOKEN": token}, data = {
    'image_url': languages,
    'link_url': job_url,
})
print(response)

response = requests.put(base_url + messages_id, headers={"PRIVATE-TOKEN": token}, data = {
    'image_url': localized,
    'link_url': job_url,
})
print(response)
