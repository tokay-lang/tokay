# Value

# Binary operation conversions

This is how Tokay builtin values are converted during binary operations.

|  + * - /  | **void** | **null** | **bool** | **int** | **float** | **addr** | **str** | **dict** | **list**
| --------- | -------- | -------- | -------- | ------- | --------- | -------- | ------- | -------- | --------
| **void**  |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **null**  |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **bool**  |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **int**   |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **float** |   float  |   float  |   float  |   float |   float   |   addr   |   str   |   dict   |   list
| **addr**  |   addr   |   addr   |   addr   |   addr  |   addr    |   addr   |   str   |   dict   |   list
| **str**   |   str    |   str    |   str    |   str   |   str     |   str    |   str   |   dict   |   list
| **dict**  |   dict   |   dict   |   dict   |   dict  |   dict    |   dict   |   dict  |   dict   |   list
| **list**  |   list   |   list   |   list   |   list  |   list    |   list   |   list  |   list   |   list
