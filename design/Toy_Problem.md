# Toy Problem

This is "Guy Steele's favorite toy problem".

Array of integers representing a bar chart / histogram.

Now consider the histogram as terrain, and let water rain on it.

How much water will it hold?

     define histogramWater = [ :array |
         [array, array prefix: #max, array suffix: #max]
             zip sum: [ :v left right | (left min: right) - v ]
     ]

